use crate::ast::parsed::*;
use crate::ast::verified::{CustomType, RawFunction, VExpr, VExprTyped};
use crate::symbols::{SymbolFunc, SymbolType};
use crate::types::{Type, VerifiedType};

use super::aggregate::ProgramAggregate;
use super::errors::{expression_error, SemanticError, SemanticResult};
use super::insights::Insights;
use super::locals::LocalVariables;
use super::operators::{calculate_binaryop, calculate_unaryop};
use super::resolvers::SymbolResolver;
use super::std_definitions::{get_std_function_raw, get_std_method, is_std_function};

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

pub struct ExpressionsVerifier<'a, 'b, 'c, 'i, 'l> {
    func: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
    locals: &'l LocalVariables,
    insights: &'i Insights,
    type_resolver: SymbolResolver<'c, SymbolType>,
    func_resolver: SymbolResolver<'c, SymbolFunc>,
}

impl<'a, 'b, 'c, 'i, 'l> ExpressionsVerifier<'a, 'b, 'c, 'i, 'l> {
    pub fn new(
        func: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        locals: &'l LocalVariables,
        insights: &'i Insights,
        type_resolver: SymbolResolver<'c, SymbolType>,
        func_resolver: SymbolResolver<'c, SymbolFunc>,
    ) -> ExpressionsVerifier<'a, 'b, 'c, 'i, 'l> {
        ExpressionsVerifier { func, aggregate, locals, insights, func_resolver, type_resolver }
    }

    fn resolve_func(&self, name: &str) -> Result<&'b RawFunction, String> {
        let func = (self.func_resolver)(name)?;
        // TODO: check for errors to be sure
        // Resolver oly returns verified functions so it is safe to unwrap from aggregate
        Ok(self.aggregate.functions.get(&func).unwrap())
    }

    fn resolve_method(&self, t: &SymbolType, method: &str) -> Result<&'b RawFunction, String> {
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
                    self.locals.get_variable(i).map_err(&with_expr)?;
                if self.insights.is_uninitialized(i) {
                    return expression_error!(expr, "Variable `{}` might be uninitialized here", i);
                }

                if_as_expected(expected, identifier_type, VExpr::GetVar(real_name))
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
                match expected {
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
            Expr::Nil => todo!("Expression Nil is not yet done!"),
        }
    }

    fn calculate_function_call(
        &self,
        original: &ExprWithPos,
        raw_called: &'b RawFunction,
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
}
