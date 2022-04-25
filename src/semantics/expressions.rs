use std::collections::HashMap;

use crate::ast::*;
use crate::loader::ModuleAlias;
use crate::types::Type;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::annotations::CustomType;
use super::light_ast::{LExpr, LExprTyped};
use super::operators::{calculate_binaryop, calculate_unaryop};
use super::resolvers::{NameResolver, SymbolResolver};
use super::std_definitions::{get_std_function_raw, get_std_method, is_std_function};
use super::symbols::{SymbolFunc, SymbolType};

fn if_as_expected(expected: Option<&Type>, real: &Type, le: LExpr) -> LExprTyped {
    if expected.is_some() && expected.unwrap() != real {
        panic!("Expected type {:?} but got {:?}", expected.unwrap(), real);
    } else {
        LExprTyped { expr: le, expr_type: real.clone() }
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

    pub fn add_variable(&mut self, name: String, t: Type) {
        if self.variables_types.contains_key(&name) {
            panic!("Variable {} already declared", name);
        }
        self.variables_types.insert(name, t);
    }
    
    fn resolve_type(&self, name: &String) -> &'b CustomType {
        self.aggregate.types.get(&(self.type_resolver)(name)).unwrap()
    }

    fn resolve_func(&self, name: &String) -> &'b RawFunction {
        self.aggregate.functions.get(&(self.func_resolver)(name)).unwrap()
    }

    fn resolve_method(&self, t: &SymbolType, method: &String) -> &'b RawFunction {
        let method_func: SymbolFunc = t.method(method);
        self.aggregate.functions.get(&method_func).unwrap()
    }

    fn resolve_field<'q>(&self, t: &'q CustomType, f: &String) -> &'q Type {
        let field_type = t.fields.iter().find(|(name, t)| *name==f);
        match field_type {
            Some((_, t)) => t,
            None => panic!("No field {} in type {:?}", f, t.name),
        }
    }

    fn err_prefix(&self) -> String {
        format!("In file {}: ", self.module)
    }

    // fn calculate_vec(&self, items: &mut Vec<Expr>) -> SemanticResult<Vec<Type>> {
    //     let calculated_items = items.iter_mut().map(|item| self.calculate_and_annotate(item));
    //     let unwrapped_items: SemanticResult<Vec<Type>> = calculated_items.collect();
    //     Ok(unwrapped_items?)
    // }

    pub fn calculate(&self, expr: &ExprWithPos, expected: Option<&Type>) -> LExprTyped {
        match &expr.expr {
            Expr::Int(i) => if_as_expected(expected, &Type::Int, LExpr::Int(*i)),
            Expr::Float(f) => if_as_expected(expected, &Type::Float, LExpr::Float(*f)),
            Expr::Bool(b) => if_as_expected(expected, &Type::Bool, LExpr::Bool(*b)),
            Expr::String(s) => if_as_expected(expected, &Type::String, LExpr::String(s.clone())),

            Expr::Identifier(i) => {
                let identifier_type = self
                    .variables_types
                    .get(i)
                    .expect(&format!("No identifier {} found", i));
                if_as_expected(expected, identifier_type, LExpr::GetVar(i.clone()))
            }
            Expr::This => {
                if self.scope.method_of.is_none() {
                    panic!("Using \"this\" is not allowed outside of methods");
                }
                let identifier_type = self
                    .variables_types
                    .get("name")
                    .expect("This is not in scope of method!");
                if_as_expected(expected, identifier_type, LExpr::GetVar("this".into()))
            }

            Expr::UnaryOp { op, operand } => {
                let operand = self.calculate(&operand, None);
                calculate_unaryop(&op, operand)
            }
            Expr::BinOp { left, right, op } => calculate_binaryop(
                op,
                self.calculate(&left, None),
                self.calculate(&right, None),
            ),

            Expr::FunctionCall { function, args } => {
                if is_std_function(function) {
                    let std_raw = get_std_function_raw(function);
                    self.calculate_function_call(&std_raw, expected, &args, None)
                } else {
                    let raw_called = self.resolve_func(&function);
                    self.calculate_function_call(&raw_called, expected, &args, None)
                }
            }
            Expr::MethodCall { object, method, args } => {
                let object = self.calculate(object, None);
                let object_type = object.expr_type.clone();
                let std_method: Box<RawFunction>;
                let raw_method = match &object_type {
                    Type::Bool | Type::Int | Type::Float | Type::String => {
                        std_method = Box::new(get_std_method(&object_type, method));
                        std_method.as_ref()
                    }
                    Type::Ident(_) => {
                        let object_symbol: SymbolType = object_type.into();
                        self.resolve_method(&object_symbol, method)
                    }
                    t => panic!("Methods are not done for {:?}", t),
                };
                // TODO: check if maybe type
                // TODO: check if tuple type
                self.calculate_function_call(&raw_method, expected, &args, Some(object))
            }
            Expr::OwnMethodCall { method, args } => {
                if self.scope.method_of.is_none() {
                    panic!("Calling own method outside of method scope!");
                }
                // TODO: review exprwithpos for this, maybe too strange tbh
                let this_object = self.calculate(
                    &ExprWithPos { expr: Expr::This, pos_first: 0, pos_last: 0 },
                    expected,
                );
                let raw_method =
                    self.resolve_method(self.scope.method_of.as_ref().unwrap(), method);
                self.calculate_function_call(&raw_method, expected, &args, Some(this_object))
            }
            Expr::NewClassInstance { typename, args } => {
                let raw_type = self.resolve_type(&typename);
                let raw_constructor = self.resolve_method(&raw_type.name, typename);
                self.calculate_function_call(&raw_constructor, expected, &args, None)
            }

            Expr::TupleValue(items) => {
                let item_types: Vec<Type>;
                let calculated: Vec<LExprTyped>;
                match expected {
                    None => {
                        calculated = items.iter().map(|item| self.calculate(item, None)).collect();
                        item_types = calculated.iter().map(|item| item.expr_type.clone()).collect();
                    }
                    Some(Type::Tuple(expected_item_types)) => {
                        calculated = items
                            .iter()
                            .zip(expected_item_types)
                            .map(|(item, item_type)| self.calculate(item, Some(item_type)))
                            .collect();
                        item_types = expected_item_types.clone();
                    }
                    Some(t) => panic!("Tuple value expected, but got {:?}", t),
                }
                LExprTyped {
                    expr: LExpr::TupleValue(calculated),
                    expr_type: Type::Tuple(item_types),
                }
            }
            // Expr::ListValue(items) => {
            //     if items.len() == 0 {
            //         // TODO: tests for anonymous type (in let and in methods)
            //         Type::List(Box::new(Type::Anonymous))
            //     } else {
            //         let calculated_items = self.calculate_vec(items)?;
            //         // Check for maybe types required, but this is not for now
            //         if calculated_items.windows(2).any(|pair| pair[0] != pair[1]) {
            //             panic!(
            //                 "{} list items have different types: {:?}",
            //                 self.err_prefix(),
            //                 calculated_items
            //             );
            //         }
            //         Type::List(Box::new(calculated_items[0].clone()))
            //     }
            // }
            Expr::ListAccess { list, index } => {
                let calculated_object = self.calculate(&list, None);
                
                match calculated_object.expr_type.clone() {
                    Type::Tuple(item_types) => {
                        let index_value: usize;
                        match index.expr {
                            Expr::Int(i) if i >= item_types.len() as i64 => {
                                panic!("Tuple index out of bounds: {} >= {}", i, item_types.len());
                            }
                            Expr::Int(i) => {
                                index_value = i as usize;
                            }
                            _ => panic!("Only integer allowed in tuple access!"),
                        }
                        return if_as_expected(
                            expected,
                            &item_types[index_value],
                            LExpr::GetTupleItem {
                                tuple: Box::new(calculated_object),
                                index: index_value,
                            },
                        );
                    }
                    Type::List(_) => todo!("Did not implemented lists yet!"),
                    t => panic!("Did not expected to have type {:?} here", t),
                }
            }
            Expr::FieldAccess { object, field } => {
                // TODO: implement something for built-in types
                let object_calculated = self.calculate(&object, None);
                match &object_calculated.expr_type {
                    Type::Ident(i) => {
                        let object_definition = self.resolve_type(i);
                        let field_type = self.resolve_field(object_definition, &field);

                        let lexpr = LExpr::AccessField { object: Box::new(object_calculated), field: field.clone() };
                        if_as_expected(expected, &field_type, lexpr)
                    },
                    _ => {
                        panic!("Error at {:?} - type {:?} has no fields", object, object_calculated.expr_type);
                    }
                }
            }
            Expr::OwnFieldAccess { field } => {
                if self.scope.method_of.is_none() {
                    panic!("Calling own method outside of method scope!");
                }
                // TODO: review exprwithpos for this, maybe too strange tbh
                let this_object = self.calculate(
                    &ExprWithPos { expr: Expr::This, pos_first: 0, pos_last: 0 },
                    expected,
                );
                let object_symbol: SymbolType = this_object.expr_type.clone().into();
                let object_definition = self.aggregate.types.get(&object_symbol).unwrap();
                let field_type = self.resolve_field(object_definition, &field);

                let lexpr = LExpr::AccessField { object: Box::new(this_object), field: field.clone() };
                if_as_expected(expected, &field_type, lexpr)
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

            
            e => todo!("Expressions {:?} is not yet done!", e),
        }
    }

    fn calculate_function_call(
        &self,
        raw_called: &'b RawFunction,
        expected_return: Option<&Type>,
        given_args: &Vec<ExprWithPos>,
        implicit_this: Option<LExprTyped>,
    ) -> LExprTyped {
        let expected_args: &[Type] = if implicit_this.is_some() {
            &raw_called.args.types[1..]
        } else {
            &raw_called.args.types[..]
        };

        if given_args.len() != expected_args.len() {
            panic!(
                "{} Function {:?} expects {} arguments, but {} given",
                self.err_prefix(),
                raw_called.name,
                expected_args.len(),
                given_args.len(),
            );
        }

        let mut processed_args: Vec<LExprTyped> = given_args
            .iter()
            .zip(expected_args.iter())
            .map(|(arg, expected_type)| self.calculate(arg, Some(expected_type)))
            .collect();
        if let Some(this_object) = implicit_this {
            processed_args.insert(0, this_object);
        }

        let lexpr_call = LExpr::CallFunction {
            name: raw_called.name.clone(),
            return_type: raw_called.return_type.clone(),
            args: processed_args,
        };

        if_as_expected(
            expected_return,
            &raw_called.return_type,
            lexpr_call,
        )
    }

    // fn calculate_access_by_index(
    //     &self,
    //     list: &mut Box<Expr>,
    //     index: &mut Box<Expr>,
    // ) -> Result<Type, String> {
    //     let list_type = self.calculate_and_annotate(list.as_mut())?;
    //     match list_type {
    //         Type::List(item) => match self.calculate_and_annotate(index.as_mut())? {
    //             Type::Int => Ok(item.as_ref().clone()),
    //             t => sem_err!("List index must be int, but got {:?} in {:?}", t, index),
    //         },
    //         Type::Tuple(items) => match index.as_ref() {
    //             Expr::Int(i) => {
    //                 let item = items.get(*i as usize);
    //                 if item.is_some() {
    //                     Ok(item.unwrap().clone())
    //                 } else {
    //                     sem_err!(
    //                         "{} Out of bounds index in {:?}",
    //                         self.err_prefix(),
    //                         list.as_ref()
    //                     )
    //                 }
    //             }
    //             _ => sem_err!("Not int for tuple access in {:?}", list.as_ref()),
    //         },
    //         _ => sem_err!(
    //             "Expected tuple or list for index access, got {:?} in {:?}",
    //             list_type,
    //             list.as_ref()
    //         ),
    //     }
    // }
}
