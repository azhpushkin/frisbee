use std::collections::HashMap;

use crate::ast::*;
use crate::loader::ModuleAlias;
use crate::semantics::errors::expression_error;
use crate::types::Type;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::annotations::CustomType;
use super::errors::SemanticResult;
use super::light_ast::{LExpr, LExprTyped};
use super::operators::{calculate_binaryop, calculate_unaryop};
use super::resolvers::{NameResolver, SymbolResolver};
use super::std_definitions::{get_std_function_raw, get_std_method, is_std_function};
use super::symbols::{SymbolFunc, SymbolType};

fn if_as_expected(
    original: &ExprWithPos,
    expected: Option<&Type>,
    real: &Type,
    le: LExpr,
) -> SemanticResult<LExprTyped> {
    if expected.is_some() && expected.unwrap() != real {
        expression_error!(
            original,
            "Expected type {} but got {}",
            expected.unwrap(),
            real
        )
    } else {
        Ok(LExprTyped { expr: le, expr_type: real.clone() })
    }
}

pub struct LightExpressionsGenerator<'a, 'b, 'c> {
    module: ModuleAlias,
    scope: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
    variables_types: HashMap<String, Type>,
    func_resolver: SymbolResolver<'c, SymbolFunc>,
    type_resolver: SymbolResolver<'c, SymbolType>,
}

impl<'a, 'b, 'c> LightExpressionsGenerator<'a, 'b, 'c> {
    pub fn new<'d>(
        scope: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        resolver: &'d NameResolver,
    ) -> LightExpressionsGenerator<'a, 'b, 'c>
    where
        'a: 'c,
        'd: 'c,
    {
        LightExpressionsGenerator {
            module: scope.defined_at.clone(),
            scope,
            aggregate,
            variables_types: HashMap::new(),
            func_resolver: resolver.get_functions_resolver(&scope.defined_at),
            type_resolver: resolver.get_typenames_resolver(&scope.defined_at),
        }
    }

    pub fn add_variable(&mut self, name: String, t: Type) -> Result<(), String> {
        if self.variables_types.contains_key(&name) {
            return Err(format!(
                "Variable {} defined multiple times in function {}",
                name, self.scope.short_name
            ));
        }
        self.variables_types.insert(name, t);
        Ok(())
    }

    fn resolve_func(&self, name: &String) -> &'b RawFunction {
        self.aggregate.functions.get(&(self.func_resolver)(name)).unwrap()
    }

    fn resolve_method(&self, t: &SymbolType, method: &String) -> &'b RawFunction {
        let method_func: SymbolFunc = t.method(method);
        self.aggregate.functions.get(&method_func).unwrap()
    }

    fn resolve_field<'q>(&self, t: &'q CustomType, f: &String) -> &'q Type {
        let field_type = t.fields.iter().find(|(name, _)| *name == f);
        match field_type {
            Some((_, t)) => t,
            None => panic!("No field {} in type {:?}", f, t.name),
        }
    }

    pub fn calculate(
        &self,
        expr: &ExprWithPos,
        expected: Option<&Type>,
    ) -> SemanticResult<LExprTyped> {
        match &expr.expr {
            Expr::Int(i) => if_as_expected(expr, expected, &Type::Int, LExpr::Int(*i)),
            Expr::Float(f) => if_as_expected(expr, expected, &Type::Float, LExpr::Float(*f)),
            Expr::Bool(b) => if_as_expected(expr, expected, &Type::Bool, LExpr::Bool(*b)),
            Expr::String(s) => {
                if_as_expected(expr, expected, &Type::String, LExpr::String(s.clone()))
            }

            Expr::Identifier(i) => {
                let identifier_type = self
                    .variables_types
                    .get(i)
                    .expect(&format!("No identifier {} found", i));
                if_as_expected(expr, expected, identifier_type, LExpr::GetVar(i.clone()))
            }
            Expr::This => {
                if self.scope.method_of.is_none() {
                    return expression_error!(
                        expr,
                        "Using \"this\" is not allowed outside of methods"
                    );
                }
                let obj_type: Type = self.scope.method_of.as_ref().unwrap().into();
                if_as_expected(expr, expected, &obj_type, LExpr::GetVar("this".into()))
            }

            Expr::UnaryOp { op, operand } => {
                let operand = self.calculate(&operand, None)?;
                Ok(calculate_unaryop(&op, operand))
            }
            Expr::BinOp { left, right, op } => Ok(calculate_binaryop(
                op,
                self.calculate(&left, None)?,
                self.calculate(&right, None)?,
            )),

            Expr::FunctionCall { function, args } => {
                if is_std_function(function) {
                    let std_raw = get_std_function_raw(function);
                    self.calculate_function_call(expr, &std_raw, expected, &args, None)
                } else {
                    let raw_called = self.resolve_func(&function);
                    self.calculate_function_call(expr, &raw_called, expected, &args, None)
                }
            }
            Expr::MethodCall { object, method, args } => {
                let le_object = self.calculate(object, None)?;
                let object_type = le_object.expr_type.clone();
                let std_method: Box<RawFunction>;
                let raw_method = match &object_type {
                    Type::Tuple(..) => return expression_error!(expr, "Tuples have no methods"),
                    Type::Maybe(..) => {
                        return expression_error!(
                            expr,
                            "Use ?. operator to access methods for Maybe type",
                        );
                    }

                    Type::Ident(_) => {
                        let object_symbol: SymbolType = object_type.into();
                        self.resolve_method(&object_symbol, method)
                    }
                    t => {
                        std_method = Box::new(get_std_method(&t, method));
                        std_method.as_ref()
                    }
                };
                // TODO: check if maybe type
                // TODO: check if tuple type
                self.calculate_function_call(expr, &raw_method, expected, &args, Some(le_object))
            }
            Expr::OwnMethodCall { method, args } => {
                if self.scope.method_of.is_none() {
                    return expression_error!(expr, "Calling own method outside of method scope!");
                }
                // TODO: review exprwithpos for this, maybe too strange tbh
                let this_object = self.calculate(
                    &ExprWithPos { expr: Expr::This, pos_first: 0, pos_last: 0 },
                    None,
                )?;
                let raw_method =
                    self.resolve_method(self.scope.method_of.as_ref().unwrap(), method);
                self.calculate_function_call(expr, &raw_method, expected, &args, Some(this_object))
            }
            Expr::NewClassInstance { typename, args } => {
                let symbol = &(self.type_resolver)(typename);
                let raw_type = &self.aggregate.types[&symbol];
                let raw_constructor = self.resolve_method(&raw_type.name, typename);
                self.calculate_function_call(expr, &raw_constructor, expected, &args, None)
            }

            Expr::TupleValue(items) => {
                let item_types: Vec<Type>;
                let calculated: Vec<LExprTyped>;
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
                Ok(LExprTyped {
                    expr: LExpr::TupleValue(calculated),
                    expr_type: Type::Tuple(item_types),
                })
            }
            Expr::ListValue(items) if items.is_empty() => match expected {
                // Case when list is empty, so expected will be always OK if it is list
                Some(Type::List(item_type)) => {
                    let item_type = item_type.as_ref().clone();

                    Ok(LExprTyped {
                        expr: LExpr::ListValue { item_type, items: vec![] },
                        expr_type: expected.unwrap().clone(),
                    })
                }
                Some(t) => {
                    return expression_error!(expr, "Unexpected list value (expected {})", t)
                }
                None => return expression_error!(expr, "Can't figure out list type over here!"),
            },
            Expr::ListValue(items) => {
                let expected_item_type = match expected {
                    None => None,
                    Some(Type::List(item_type)) => Some(item_type.as_ref().clone()),
                    Some(t) => {
                        return expression_error!(expr, "Unexpected list value (expected {})", t)
                    }
                };
                let calculated_items: SemanticResult<Vec<_>> = items
                    .iter()
                    .map(|expr| self.calculate(expr, expected_item_type.as_ref()))
                    .collect();
                let calculated_items = calculated_items?;

                let item_type: Type;
                if expected_item_type.is_none() {
                    item_type = calculated_items[0].expr_type.clone();
                    for LExprTyped { expr_type, .. } in &calculated_items[1..] {
                        if expr_type != &item_type {
                            return expression_error!(
                                expr,
                                "Item types mismatch in a list: {} and {}",
                                item_type,
                                expr_type
                            );
                        }
                    }
                } else {
                    item_type = expected_item_type.unwrap().clone()
                };

                Ok(LExprTyped {
                    expr: LExpr::ListValue {
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
                let object_calculated = self.calculate(&object, None)?;
                match &object_calculated.expr_type {
                    Type::Ident(_) => {
                        let type_symbol: SymbolType = object_calculated.expr_type.clone().into();

                        let object_definition = &self.aggregate.types[&type_symbol];
                        let field_type = self.resolve_field(object_definition, &field);

                        let lexpr = LExpr::AccessField {
                            object: Box::new(object_calculated),
                            field: field.clone(),
                        };
                        if_as_expected(expr, expected, &field_type, lexpr)
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
                if self.scope.method_of.is_none() {
                    return expression_error!(expr, "Accessing own field outside of method scope!");
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

                let object_symbol: SymbolType = this_object.expr_type.clone().into();
                let object_definition = self.aggregate.types.get(&object_symbol).unwrap();
                let field_type = self.resolve_field(object_definition, &field);

                let lexpr =
                    LExpr::AccessField { object: Box::new(this_object), field: field.clone() };
                if_as_expected(expr, expected, &field_type, lexpr)
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
        expected_return: Option<&Type>,
        given_args: &Vec<ExprWithPos>,
        implicit_this: Option<LExprTyped>,
    ) -> SemanticResult<LExprTyped> {
        let expected_args: &[Type] = if implicit_this.is_some() {
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

        let processed_args: SemanticResult<Vec<LExprTyped>> = given_args
            .iter()
            .zip(expected_args.iter())
            .map(|(arg, expected_type)| self.calculate(arg, Some(expected_type)))
            .collect();
        let mut processed_args = processed_args.unwrap();
        if let Some(this_object) = implicit_this {
            processed_args.insert(0, this_object);
        }

        let lexpr_call = LExpr::CallFunction {
            name: raw_called.name.clone(),
            return_type: raw_called.return_type.clone(),
            args: processed_args,
        };

        if_as_expected(
            original,
            expected_return,
            &raw_called.return_type,
            lexpr_call,
        )
    }

    fn calculate_access_by_index(
        &self,
        original: &ExprWithPos,
        object: &ExprWithPos,
        index: &ExprWithPos,
        expected: Option<&Type>,
    ) -> SemanticResult<LExprTyped> {
        let calculated_object = self.calculate(&object, None)?;

        match calculated_object.expr_type.clone() {
            Type::Tuple(item_types) => {
                let index_value: usize;
                match index.expr {
                    Expr::Int(i) if i >= item_types.len() as i64 => {
                        return expression_error!(
                            index,
                            "Index of tuple is out of bounds (must be between 0 and {})",
                            item_types.len()
                        );
                    }
                    Expr::Int(i) => {
                        index_value = i as usize;
                    }
                    _ => return expression_error!(index, "Only integer allowed in tuple access!"),
                }
                return if_as_expected(
                    original,
                    expected,
                    &item_types[index_value],
                    LExpr::AccessTupleItem {
                        tuple: Box::new(calculated_object),
                        index: index_value,
                    },
                );
            }
            Type::List(inner) => {
                let calculated_index = self.calculate(index, Some(&Type::Int))?;
                let new_expr = LExpr::AccessListItem {
                    list: Box::new(calculated_object),
                    index: Box::new(calculated_index),
                };
                return if_as_expected(original, expected, inner.as_ref(), new_expr);
            }
            t => expression_error!(
                object,
                "Only lists and tuples implement index access (got {})",
                t
            ),
        }
    }
}
