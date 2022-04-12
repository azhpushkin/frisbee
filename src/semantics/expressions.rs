use std::collections::HashMap;

use crate::ast::*;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::annotations::CustomType;
use super::light_ast::{LExpr, LExprTyped};
use super::operators::{calculate_unaryop, calculate_binaryop};
use super::resolvers::{NameResolver, SymbolResolver};
use super::symbols::{SymbolFunc, SymbolType};

pub fn if_as_expected(e: Option<&Type>, t: &Type, le: LExpr) -> LExprTyped {
    if e.is_some() && e.unwrap() != t {
        panic!("Expected type {:?} but got {:?}", e.unwrap(), t);
    } else {
        LExprTyped { expr: le, expr_type: t }
    }
}

pub struct LightExpressionsGenerator<'a, 'b, 'c> {
    module: ModulePathAlias,
    scope: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
    variables_types: HashMap<String, Type>,
    func_resolver: SymbolResolver<'c, SymbolFunc>,
    type_resolver: SymbolResolver<'c, SymbolType>,
}

impl<'a, 'b, 'c> LightExpressionsGenerator<'a, 'b, 'c> {
    pub fn new(
        scope: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        resolver: &'c NameResolver,
    ) -> LightExpressionsGenerator<'a, 'b, 'c> {
        let module = scope.defined_at.clone();
        LightExpressionsGenerator {
            module,
            scope,
            aggregate,
            variables_types: HashMap::new(),
            func_resolver: resolver.get_functions_resolver(&module),
            type_resolver: resolver.get_typenames_resolver(&module),
        }
    }

    pub fn add_variable(&mut self, name: String, t: Type) {
        if self.variables_types.contains_key(&name) {
            panic!("Variable {} already declared", name);
        }
        self.variables_types.insert(name, t);
    }

    pub fn resolve_type(&self, name: &String) -> &'b CustomType {
        self.aggregate.types.get(&(self.type_resolver)(name)).unwrap()
    }

    pub fn resolve_func(&self, name: &String) -> &'b RawFunction {
        self.aggregate.functions.get(&(self.func_resolver)(name)).unwrap()
    }

    fn err_prefix(&self) -> String {
        format!("In file {}: ", self.module.0)
    }

    // fn calculate_vec(&self, items: &mut Vec<Expr>) -> SemanticResult<Vec<Type>> {
    //     let calculated_items = items.iter_mut().map(|item| self.calculate_and_annotate(item));
    //     let unwrapped_items: SemanticResult<Vec<Type>> = calculated_items.collect();
    //     Ok(unwrapped_items?)
    // }

    pub fn calculate(&self, expr: &Expr, expected: Option<&Type>) -> LExprTyped {
        match expr {
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

            // Expr::FunctionCall { function, args } => {
            //     let function_signature = self.get_function_signature(function)?;
            //     let rettype = self.check_function_call(function_signature, args)?;
            //     *expr = Expr::FunctionCallQualified {
            //         module: self.get_function_file(function),
            //         function: std::mem::take(function),
            //         args: std::mem::take(args),
            //     };
            //     rettype
            // }

            
            // Expr::TupleValue(items) => Type::Tuple(self.calculate_vec(items)?),
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

            // Expr::ListAccess { list, index } => self.calculate_access_by_index(list, index)?,

            // Expr::NewClassInstance { typename, args } => {
            //     let class_signature = self.get_class_signature(typename)?;
            //     if class_signature.is_active {
            //         panic!("{} You must call spawn for {}", self.err_prefix(), typename);
            //     }

            //     let constuctor =
            //         class_signature.methods.get(typename).expect("Constructor not found");
            //     self.check_function_call(constuctor, args)?
            // }
            // Expr::SpawnActive { typename, args } => {
            //     let class_signature = self.get_class_signature(typename)?;

            //     if !class_signature.is_active {
            //         panic!("{} Cant spawn passive {}!", self.err_prefix(), typename);
            //     }
            //     let constuctor =
            //         class_signature.methods.get(typename).expect("Constructor not found");
            //     self.check_function_call(constuctor, args)?
            // }
            

            // Expr::MethodCall { object, method, args } => {
            //     // TODO: implement something for built-in types
            //     let obj_type = self.calculate_and_annotate(object)?;
            //     match &obj_type {
            //         Type::IdentQualified(alias, name) => {
            //             let method = self.get_type_method(alias, name, method)?;
            //             self.check_function_call(method, args)?
            //         }
            //         Type::Ident(..) => panic!("TypeIdent should not be present here!"),
            //         Type::Maybe(_) => panic!("Not implemented for maybe yet!"), // implement ?.
            //         t => {
            //             let method = get_std_method(t, method)?;
            //             self.check_function_call(&method, args)?
            //         }
            //     }
            // }

            // Expr::FieldAccess { object, field } => {
            //     // TODO: implement something for built-in types
            //     let obj_type = self.calculate_and_annotate(object)?;
            //     match &obj_type {
            //         Type::IdentQualified(alias, name) => {
            //             let field_type = self.get_type_field(alias, name, field)?;
            //             field_type.clone()
            //         }
            //         Type::Ident(..) => panic!("TypeIdent should not be present here!"),
            //         _ => {
            //             panic!("Error at {:?} - type {:?} has no fields", object, obj_type)
            //         }
            //     }
            // }
            // Expr::OwnMethodCall { .. } | Expr::OwnFieldAccess { .. } if self.scope.is_none() => {
            //     panic!("{} Using @ is not allowed in functions", self.err_prefix())
            // }
            // Expr::OwnMethodCall { method, args } => {
            //     let method_signature =
            //         self.get_type_method(&self.file_name, self.scope.as_ref().unwrap(), method)?;
            //     self.check_function_call(method_signature, args)?
            // }
            // Expr::OwnFieldAccess { field } => {
            //     let field_type =
            //         self.get_type_field(&self.file_name, &self.scope.as_ref().unwrap(), field)?;
            //     field_type.clone()
            // }
            e => todo!("Expressions {:?} is not yet done!", e),
        }
    }

    // fn check_function_call(
    //     &self,
    //     function: &FunctionSignature,
    //     args: &mut Vec<Expr>,
    // ) -> SemanticResult<Type> {
    //     if function.args.len() != args.len() {
    //         panic!(
    //             "{} Wrong amount of arguments at {:?}, expected {}",
    //             self.err_prefix(),
    //             args,
    //             function.args.len()
    //         );
    //     }

    //     for (expected_arg, arg_expr) in function.args.iter().zip(args.iter_mut()) {
    //         let expr_type = self.calculate_and_annotate(arg_expr)?;
    //         // TODO: this is wrong check of type correctness, but it works for now
    //         if !are_types_same_or_maybe(&expected_arg.1, &expr_type) {
    //             panic!(
    //                 "Wrong type for argument {}, expected {:?}, got {:?} ({:?})",
    //                 expected_arg.0, expected_arg.1, expr_type, arg_expr
    //             );
    //         }
    //     }

    //     Ok(function.rettype.clone())
    // }

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
