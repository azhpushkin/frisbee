use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::parsed::*;
use crate::ast::verified::{CustomType, RawFunction, RawOperator, VExpr, VExprTyped};
use crate::symbols::{SymbolFunc, SymbolType};
use crate::types::{Type, VerifiedType};

use super::aggregate::ProgramAggregate;
use super::errors::{expression_error, SemanticError, SemanticResult};
use super::insights::Insights;
use super::locals::LocalVariables;
use super::operators::{calculate_binaryop, calculate_unaryop, wrap_binary};
use super::resolvers::SymbolResolver;
use super::std_definitions::{get_std_function_raw, get_std_method, is_std_function};


macro_rules! unwrapped_maybe_err {
    ($expr:expr) => {
        (match $expr {
            Some(Type::Maybe(i)) => Some(i.as_ref()),
            t => t
        })
    };
}

fn if_as_expected(
    expected: Option<&VerifiedType>,
    calculated: &VerifiedType,
    expr: VExpr,
) -> Result<VExprTyped, String> {
    let expr = VExprTyped { expr, expr_type: calculated.clone() };
    match expected {
        Some(t) if calculated == t => Ok(expr),
        Some(Type::Maybe(inner)) if calculated == inner.as_ref() => Ok(VExprTyped {
            expr: VExpr::TupleValue(vec![
                VExprTyped { expr: VExpr::Bool(true), expr_type: Type::Int },
                expr,
            ]),
            expr_type: expected.unwrap().clone(),
        }),
        Some(_) => Err(format!(
            "Expected type `{}` but got `{}`",
            expected.unwrap(),
            calculated
        )),
        None => Ok(expr),
    }
}

// In fact, func, aggregate and resolver lifetimes are somewhat different
// (see resolvers code for more details), but it is fine to generalize them here
pub struct ExpressionsVerifier<'a, 'i> {
    func: &'a RawFunction,
    aggregate: &'a ProgramAggregate,
    locals: Rc<RefCell<LocalVariables>>,
    insights: &'i Insights,
    type_resolver: SymbolResolver<'a, SymbolType>,
    func_resolver: SymbolResolver<'a, SymbolFunc>,
    pub required_temps: RefCell<Vec<(String, VExprTyped)>>,
}

impl<'a, 'i> ExpressionsVerifier<'a, 'i> {
    pub fn new(
        func: &'a RawFunction,
        aggregate: &'a ProgramAggregate,
        locals: Rc<RefCell<LocalVariables>>,
        insights: &'i Insights,
        type_resolver: SymbolResolver<'a, SymbolType>,
        func_resolver: SymbolResolver<'a, SymbolFunc>,
    ) -> Self {
        ExpressionsVerifier {
            func,
            aggregate,
            locals,
            insights,
            func_resolver,
            type_resolver,
            required_temps: RefCell::new(vec![]),
        }
    }

    fn resolve_func(&self, name: &str) -> Result<&'a RawFunction, String> {
        let func = (self.func_resolver)(name)?;
        // TODO: check for errors to be sure
        // Resolver oly returns verified functions so it is safe to unwrap from aggregate
        Ok(self.aggregate.functions.get(&func).unwrap())
    }

    fn resolve_method(&self, t: &SymbolType, method: &str) -> Result<&'a RawFunction, String> {
        let method_func: SymbolFunc = t.method(method);
        self.aggregate
            .functions
            .get(&method_func)
            .ok_or_else(|| format!("No method {} in type {}", method, t))
    }

    fn resolve_field<'q>(&self, t: &'q CustomType, f: &str) -> Result<&'q VerifiedType, String> {
        t.fields
            .iter()
            .find(|(name, _)| *name == f)
            .map(|(_, t)| t)
            .ok_or_else(|| format!("No field {} in type {}", f, t.name))
    }

    fn request_temp(&self, expr_to_store: VExprTyped, seed: usize) -> String {
        let name = format!("$temp_{}", seed);
        self.required_temps.borrow_mut().push((name.clone(), expr_to_store));
        name
    }

    pub fn calculate(
        &self,
        expr: &ExprWithPos,
        expected: Option<&VerifiedType>,
    ) -> SemanticResult<VExprTyped> {
        let with_expr = SemanticError::add_expr(expr);
        match &expr.expr {
            Expr::Int(i) => if_as_expected(expected, &Type::Int, VExpr::Int(*i)).map_err(with_expr),
            Expr::Float(f) => {
                if_as_expected(expected, &Type::Float, VExpr::Float(*f)).map_err(with_expr)
            }
            Expr::Bool(b) => {
                if_as_expected(expected, &Type::Bool, VExpr::Bool(*b)).map_err(with_expr)
            }
            Expr::String(s) => {
                if_as_expected(expected, &Type::String, VExpr::String(s.clone())).map_err(with_expr)
            }

            Expr::Identifier(i) => {
                let (identifier_type, real_name) =
                    self.locals.borrow().get_variable(i).map_err(&with_expr)?;
                if self.insights.is_uninitialized(i) {
                    return expression_error!(expr, "Variable `{}` might be uninitialized here", i);
                }

                if_as_expected(expected, &identifier_type, VExpr::GetVar(real_name))
                    .map_err(with_expr)
            }
            Expr::This => match &self.func.method_of {
                Some(t) => if_as_expected(
                    expected,
                    &Type::Custom(t.clone()),
                    VExpr::GetVar("this".into()),
                )
                .map_err(with_expr),
                None => expression_error!(expr, "Using \"this\" is not allowed outside of methods"),
            },

            Expr::UnaryOp { op, operand } => {
                let operand = self.calculate(operand, None)?;
                let unary_res = calculate_unaryop(op, operand).map_err(&with_expr)?;
                if_as_expected(expected, &unary_res.expr_type, unary_res.expr).map_err(&with_expr)
            }
            Expr::BinOp { left, right, op }
                if op == &BinaryOp::IsEqual || op == &BinaryOp::IsNotEqual =>
            {
                let mut res = self.calculate_equality(left, right)?;
                if matches!(op, BinaryOp::IsNotEqual) {
                    res = calculate_unaryop(&UnaryOp::Not, res).map_err(&with_expr)?;
                }
                if_as_expected(expected, &res.expr_type, res.expr).map_err(&with_expr)
            }
            Expr::BinOp { left, right, op } if op == &BinaryOp::Elvis => {
                let left = self.calculate(left, None)?;
                let right = self.calculate(right, None)?;
                self.calculate_elvis(left, right, expr.pos_first).map_err(&with_expr)
            }
            Expr::BinOp { left, right, op } => {
                let binary_res = calculate_binaryop(
                    op,
                    self.calculate(left, None)?,
                    self.calculate(right, None)?,
                )
                .map_err(&with_expr)?;
                if_as_expected(expected, &binary_res.expr_type, binary_res.expr).map_err(&with_expr)
            }

            Expr::FunctionCall { function, args } => {
                if is_std_function(function) {
                    let std_raw = get_std_function_raw(function);
                    self.calculate_function_call(expr, &std_raw, expected, args, None)
                } else {
                    let raw_called =
                        self.resolve_func(function).map_err(SemanticError::add_expr(expr))?;
                    self.calculate_function_call(expr, raw_called, expected, args, None)
                }
            }
            Expr::MethodCall { object, method, args } => {
                let le_object = self.calculate(object, None)?;

                let std_method: Box<RawFunction>;
                let raw_method = match &le_object.expr_type {
                    Type::Tuple(..) => return expression_error!(expr, "Tuples have no methods"),
                    Type::Maybe(..) => {
                        return expression_error!(
                            expr,
                            "Use ?. operator to access methods for Maybe type",
                        );
                    }

                    Type::Custom(symbol_type) => {
                        self.resolve_method(&symbol_type, method).map_err(&with_expr)?
                    }
                    t => {
                        std_method = get_std_method(t, method).map_err(with_expr)?;
                        std_method.as_ref()
                    }
                };
                // TODO: check if maybe type
                self.calculate_function_call(expr, raw_method, expected, args, Some(le_object))
            }
            Expr::MaybeMethodCall { .. } => {
                todo!();
                // let ve_object = self.calculate(object, None)?;
                // let inner_type = match ve_object.expr_type {
                //     Type::Maybe(t) => t.as_ref(),
                //     _ => return expression_error!(expr, "?. operator to can only be used on Maybe types"),
                // };
            }
            Expr::OwnMethodCall { method, args } => {
                let type_of_func = match &self.func.method_of {
                    Some(t) => t,
                    _ => expression_error!(expr, "Calling own method outside of class!")?,
                };
                // TODO: review exprwithpos for this, maybe too strange tbh
                let this_object = self.calculate(
                    &ExprWithPos { expr: Expr::This, pos_first: 0, pos_last: 0 },
                    None,
                )?;
                let raw_method = self.resolve_method(type_of_func, method).map_err(with_expr)?;
                self.calculate_function_call(expr, raw_method, expected, args, Some(this_object))
            }
            Expr::NewClassInstance { typename, args } => {
                let symbol =
                    &(self.type_resolver)(typename).map_err(SemanticError::add_expr(expr))?;
                let raw_type = &self.aggregate.types[symbol];
                let raw_constructor =
                    self.resolve_method(&raw_type.name, typename).map_err(with_expr)?;
                self.calculate_function_call(expr, raw_constructor, expected, args, None)
            }

            Expr::TupleValue(items) => {
                let item_types: Vec<VerifiedType>;
                let calculated: Vec<VExprTyped>;
                match unwrapped_maybe_err!(expected) {
                    None => {
                        let calculated_result: SemanticResult<Vec<_>> =
                            items.iter().map(|item| self.calculate(item, None)).collect();
                        calculated = calculated_result?;
                        item_types = calculated.iter().map(|item| item.expr_type.clone()).collect();
                    }
                    Some(Type::Tuple(expected_item_types)) => {
                        let calculated_result: SemanticResult<Vec<_>> = items
                            .iter()
                            .zip(expected_item_types)
                            .map(|(item, item_type)| self.calculate(item, Some(item_type)))
                            .collect();
                        calculated = calculated_result?;
                        item_types = expected_item_types.clone();
                    }
                    Some(t) => {
                        return expression_error!(expr, "Unexpected tuple value (expected {})", t)
                    }
                }
                Ok(VExprTyped {
                    expr: VExpr::TupleValue(calculated),
                    expr_type: Type::Tuple(item_types),
                })
            }
            Expr::ListValue(items) if items.is_empty() => match expected {
                // Case when list is empty, so expected will be always OK if it is list
                Some(Type::List(item_type)) => {
                    let item_type = item_type.as_ref().clone();

                    Ok(VExprTyped {
                        expr: VExpr::ListValue { item_type, items: vec![] },
                        expr_type: expected.unwrap().clone(),
                    })
                }
                Some(t) => {
                    return expression_error!(expr, "Unexpected list value (expected {})", t)
                }
                None => return expression_error!(expr, "Can't figure out list type over here!"),
            },
            Expr::ListValue(items) => {
                // Due to previous check, we know that in this branch items are not empty
                let expected_item_type = match expected {
                    None => None,
                    Some(Type::List(item_type)) => Some(item_type.as_ref()),
                    Some(t) => {
                        return expression_error!(expr, "Unexpected list value (expected {})", t)
                    }
                };
                let calculated_items: SemanticResult<Vec<_>> = items
                    .iter()
                    .map(|expr| self.calculate(expr, expected_item_type))
                    .collect();
                let calculated_items = calculated_items?;

                // Check if all items are of same type using sliding window
                // (if there is just one element - window will have 0 iterations)
                let mismatched_pair =
                    calculated_items.windows(2).find(|p| p[0].expr_type != p[1].expr_type);
                if let Some(pair) = mismatched_pair {
                    return expression_error!(
                        expr,
                        "All items in list must be of same type, but both {} and {} are found",
                        pair[0].expr_type,
                        pair[1].expr_type
                    );
                }

                let item_type =
                    expected_item_type.unwrap_or(&calculated_items[0].expr_type).clone();

                Ok(VExprTyped {
                    expr: VExpr::ListValue {
                        item_type: item_type.clone(),
                        items: calculated_items,
                    },
                    expr_type: Type::List(Box::new(item_type)),
                })
            }
            Expr::ListAccess { list, index } => {
                self.calculate_access_by_index(expr, list, index, expected)
            }
            Expr::FieldAccess { object, field } => {
                let object_calculated = self.calculate(object, None)?;
                match &object_calculated.expr_type {
                    Type::Custom(type_symbol) => {
                        let object_definition = &self.aggregate.types[type_symbol];
                        let field_type =
                            self.resolve_field(object_definition, field).map_err(&with_expr)?;

                        let vexpr = VExpr::AccessField {
                            object: Box::new(object_calculated),
                            field: field.clone(),
                        };
                        if_as_expected(expected, field_type, vexpr).map_err(&with_expr)
                    }
                    _ => {
                        expression_error!(
                            expr,
                            "Accessing fields for type {} is prohobited",
                            object_calculated.expr_type
                        )
                    }
                }
            }
            Expr::OwnFieldAccess { field } => {
                let func_type = match &self.func.method_of {
                    Some(t) => t,
                    _ => expression_error!(expr, "Accessing own field outside of method func!")?,
                };
                if self.func.is_constructor && !self.insights.initialized_own_fields.contains(field)
                {
                    return expression_error!(
                        expr,
                        "Own field `{}` cannot be used before initializing",
                        field
                    );
                }
                // TODO: review exprwithpos for this, maybe too strange tbh
                let this_object = self.calculate(
                    &ExprWithPos {
                        expr: Expr::This,
                        pos_first: expr.pos_first,
                        pos_last: expr.pos_first,
                    },
                    None,
                )?;

                let object_definition = self.aggregate.types.get(func_type).unwrap();
                let field_type =
                    self.resolve_field(object_definition, field).map_err(&with_expr)?;

                let vexpr =
                    VExpr::AccessField { object: Box::new(this_object), field: field.clone() };
                if_as_expected(expected, field_type, vexpr).map_err(&with_expr)
            }

            // Expr::SpawnActive { typename, args } => {
            //     let class_signature = self.get_class_signature(typename)?;

            //     if !class_signature.is_active {
            //         panic!("{} Cant spawn passive {}!", self.err_prefix(), typename);
            //     }
            //     let constuctor =
            //         class_signature.methods.get(typename).expect("Constructor not found");
            //     self.check_function_call(constuctor, args)?
            // }
            Expr::SpawnActive { .. } => todo!("Expression SpawnActive is not yet done!"),
            Expr::Nil => match expected {
                Some(Type::Maybe(i)) => {
                    let tuple_items = vec![
                        VExprTyped { expr: VExpr::Bool(false), expr_type: Type::Bool },
                        VExprTyped {
                            expr: VExpr::Dummy(i.as_ref().clone()),
                            expr_type: i.as_ref().clone(),
                        },
                    ];
                    Ok(VExprTyped {
                        expr: VExpr::TupleValue(tuple_items),
                        expr_type: expected.unwrap().clone(),
                    })
                }
                Some(t) => {
                    return expression_error!(
                        expr,
                        "`nil` is only allowed as a maybe type (expected `{}`)",
                        t
                    )
                }
                None => {
                    return expression_error!(expr, "`nil` is not allowed here (can't derive type)")
                }
            },
        }
    }

    fn calculate_function_call(
        &self,
        original: &ExprWithPos,
        raw_called: &'a RawFunction,
        expected_return: Option<&VerifiedType>,
        given_args: &[ExprWithPos],
        implicit_this: Option<VExprTyped>,
    ) -> SemanticResult<VExprTyped> {
        // TODO: mark called function as used, strip unused functions
        let expected_args: &[VerifiedType] = if implicit_this.is_some() {
            &raw_called.args.types[1..]
        } else {
            &raw_called.args.types[..]
        };

        if given_args.len() != expected_args.len() {
            return expression_error!(
                original,
                "Function {} expects {} arguments, but {} given",
                raw_called.short_name,
                expected_args.len(),
                given_args.len(),
            );
        }

        let processed_args: SemanticResult<Vec<VExprTyped>> = given_args
            .iter()
            .zip(expected_args.iter())
            .map(|(arg, expected_type)| self.calculate(arg, Some(expected_type)))
            .collect();
        let mut processed_args = processed_args?;
        if let Some(this_object) = implicit_this {
            processed_args.insert(0, this_object);
        }

        let vexpr_call = VExpr::CallFunction {
            name: raw_called.name.clone(),
            return_type: raw_called.return_type.clone(),
            args: processed_args,
        };

        if_as_expected(expected_return, &raw_called.return_type, vexpr_call)
            .map_err(SemanticError::add_expr(original))
    }

    fn calculate_access_by_index(
        &self,
        original: &ExprWithPos,
        object: &ExprWithPos,
        index: &ExprWithPos,
        expected: Option<&VerifiedType>,
    ) -> SemanticResult<VExprTyped> {
        let calculated_object = self.calculate(object, None)?;

        match calculated_object.expr_type.clone() {
            Type::Tuple(item_types) => match index.expr {
                Expr::Int(i) if i >= item_types.len() as i64 => {
                    expression_error!(
                        index,
                        "Index of tuple is out of bounds (must be between 0 and {})",
                        item_types.len()
                    )
                }
                Expr::Int(i) => {
                    let vexpr = VExpr::AccessTupleItem {
                        tuple: Box::new(calculated_object),
                        index: i as usize,
                    };
                    if_as_expected(expected, &item_types[i as usize], vexpr)
                        .map_err(SemanticError::add_expr(original))
                }
                _ => expression_error!(index, "Only integer allowed in tuple access!"),
            },
            Type::List(inner) => {
                let calculated_index = self.calculate(index, Some(&Type::Int))?;
                let new_expr = VExpr::AccessListItem {
                    list: Box::new(calculated_object),
                    index: Box::new(calculated_index),
                };
                if_as_expected(expected, inner.as_ref(), new_expr)
                    .map_err(SemanticError::add_expr(original))
            }
            t => expression_error!(
                object,
                "Only lists and tuples implement index access (got {})",
                t
            ),
        }
    }

    fn calculate_equality(
        &self,
        left_og: &ExprWithPos,
        right_og: &ExprWithPos,
    ) -> SemanticResult<VExprTyped> {
        if left_og.expr == Expr::Nil {
            if right_og.expr == Expr::Nil {
                return Ok(VExprTyped { expr: VExpr::Bool(true), expr_type: Type::Bool });
            } else {
                return self.calculate_equality(right_og, left_og);
            }
        }
        // Now, either there is no `nil`, or only `right` is nil
        if right_og.expr == Expr::Nil {
            let left_calculated = self.calculate(left_og, None)?;
            if !matches!(&left_calculated.expr_type, &Type::Maybe(_)) {
                return expression_error!(
                    right_og,
                    "Cannot compare `nil` with type {} (must be maybe type)",
                    left_calculated.expr_type
                );
            }
            // obj == nil is the same, as (not obj[0])
            // as maybe starts with bool that indicates if value is there
            let access_flag = VExprTyped {
                expr: VExpr::AccessTupleItem { tuple: Box::new(left_calculated), index: 0 },
                expr_type: Type::Bool,
            };
            // negate the flag, so that it is the same as `not obj[0]`
            // unwrap as there is type-related errors in there expected
            return Ok(calculate_unaryop(&UnaryOp::Not, access_flag).unwrap());
        }

        // No need for any expected, as the only type that is generic is nil, and
        // we have already covered in above
        let left = self.calculate(left_og, None)?;
        let right = self.calculate(right_og, None)?;
        let is_eq_error_msg = format!(
            "Types `{}` and `{}` cannot be checked for equality",
            &left.expr_type, &right.expr_type,
        );

        // Helper closures to operate with temps
        // We are forced to store nulls to temp because single operator (Int? == Int)
        // is in fact unwrapped into two operators: (Int?[0] and Int?[1] == Int)
        // (check flag, then check value)

        let get_temp = |n: &str, t: &VerifiedType| VExprTyped {
            expr: VExpr::GetVar(n.into()),
            expr_type: Type::Maybe(Box::new(t.clone())),
        };
        let get_flag = |n, t: &VerifiedType| VExprTyped {
            expr: VExpr::AccessTupleItem { tuple: Box::new(get_temp(n, t)), index: 0 },
            expr_type: Type::Bool,
        };
        let get_value = |n, t: &VerifiedType| VExprTyped {
            expr: VExpr::AccessTupleItem { tuple: Box::new(get_temp(n, t)), index: 1 },
            expr_type: t.clone(),
        };
        let get_eq_op = |t: &VerifiedType, err_msg| match t {
            Type::Int => Ok(RawOperator::EqualInts),
            Type::Float => Ok(RawOperator::EqualFloats),
            Type::Bool => Ok(RawOperator::EqualBools),
            Type::String => Ok(RawOperator::EqualStrings),
            _ => return Err(err_msg),
        };

        match (left.expr_type.clone(), right.expr_type.clone()) {
            (Type::Maybe(left_inner), Type::Maybe(right_inner)) => {
                if left_inner != right_inner {
                    return expression_error!(left_og, "{}", is_eq_error_msg);
                }
                let op = get_eq_op(&left_inner, is_eq_error_msg)
                    .map_err(SemanticError::add_expr(left_og))?;

                let left_temp = self.request_temp(left, left_og.pos_first);
                let right_temp = self.request_temp(right, right_og.pos_first);

                let are_both_false = wrap_binary(
                    RawOperator::AndBools,
                    vec![
                        calculate_unaryop(&UnaryOp::Not, get_flag(&left_temp, &left_inner))
                            .unwrap(),
                        calculate_unaryop(&UnaryOp::Not, get_flag(&right_temp, &right_inner))
                            .unwrap(),
                    ],
                    Type::Bool,
                );
                let are_both_true = wrap_binary(
                    RawOperator::AndBools,
                    vec![get_flag(&left_temp, &left_inner), get_flag(&right_temp, &right_inner)],
                    Type::Bool,
                );
                let are_values_equal = wrap_binary(
                    op,
                    vec![get_value(&left_temp, &left_inner), get_value(&right_temp, &right_inner)],
                    left_inner.as_ref().clone(),
                );

                let if_both_true = wrap_binary(
                    RawOperator::AndBools,
                    vec![are_both_true, are_values_equal],
                    Type::Bool,
                );
                Ok(wrap_binary(
                    RawOperator::OrBools,
                    vec![are_both_false, if_both_true],
                    Type::Bool,
                ))
            }
            (Type::Maybe(left_inner), rt) => {
                if left_inner.as_ref() != &rt {
                    return expression_error!(left_og, "{}", is_eq_error_msg);
                }
                let op = get_eq_op(&left_inner, is_eq_error_msg)
                    .map_err(SemanticError::add_expr(left_og))?;

                let left_temp = self.request_temp(left, left_og.pos_first);

                let are_values_equal =
                    wrap_binary(op, vec![get_value(&left_temp, &left_inner), right], rt);
                let if_both_true = wrap_binary(
                    RawOperator::AndBools,
                    vec![get_flag(&left_temp, &left_inner), are_values_equal],
                    Type::Bool,
                );
                Ok(if_both_true)
            }
            (_, Type::Maybe(_)) => self.calculate_equality(right_og, left_og),
            (t1, t2) if t1 != t2 => {
                return expression_error!(
                    left_og,
                    "Types `{}` and `{}` cannot be checked for equality",
                    t1,
                    t2
                );
            }
            (_, _) => {
                let op = get_eq_op(&left.expr_type, is_eq_error_msg)
                    .map_err(SemanticError::add_expr(left_og))?;
                Ok(wrap_binary(op, vec![left, right], Type::Bool))
            }
        }
    }

    fn calculate_elvis(
        &self,
        left: VExprTyped,
        right: VExprTyped,
        seed: usize,
    ) -> Result<VExprTyped, String> {
        match (&left.expr_type, &right.expr_type) {
            (Type::Maybe(l), r) => {
                if l.as_ref() != r {
                    return Err(format!(
                        "Expected `{}` as right part of elvis, but got `{}`",
                        l, r
                    ));
                }
            }
            (l, _) => {
                return Err(format!(
                    "Maybe type must be left part of elvis, but got `{}`",
                    l
                ));
            }
        }
        let inner_type = right.expr_type.clone();
        let left_temp = self.request_temp(left, seed);

        let get_index_of_temp = |i| VExpr::AccessTupleItem {
            tuple: Box::new(VExprTyped {
                expr: VExpr::GetVar(left_temp.clone()),
                expr_type: Type::Maybe(Box::new(inner_type.clone())),
            }),
            index: i,
        };

        return Ok(VExprTyped {
            expr: VExpr::TernaryOp {
                condition: Box::new(VExprTyped {
                    expr: get_index_of_temp(0),
                    expr_type: Type::Bool,
                }),
                if_true: Box::new(VExprTyped { expr: get_index_of_temp(1), expr_type: inner_type.clone() }),
                if_false: Box::new(right),
            },
            expr_type: inner_type.clone(),
        });
    }
}
