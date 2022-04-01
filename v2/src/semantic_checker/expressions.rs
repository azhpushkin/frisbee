use std::collections::HashMap;

use super::modules::FunctionSignature;
use super::operators::{calculate_binaryop_type, calculate_unaryop_type};
use super::semantic_error::{SemanticResult, sem_err};
use super::std_definitions::{get_std_methods, get_std_method};
use super::type_env::TypeEnv;
use crate::ast::*;

pub struct ExprTypeChecker<'a> {
    env: &'a TypeEnv,
}

impl<'a> ExprTypeChecker<'a> {
    pub fn new(type_env: &'a TypeEnv) -> ExprTypeChecker<'a> {
        ExprTypeChecker { env: type_env }
    }

    pub fn calculate(&self, expr: &Expr) -> SemanticResult<Type> {
        match expr {
            // Primitive types, that map to basic types
            Expr::ExprInt(_) => Ok(Type::TypeInt),
            Expr::ExprString(_) => Ok(Type::TypeString),
            Expr::ExprBool(_) => Ok(Type::TypeBool),
            Expr::ExprNil => Ok(Type::TypeNil),
            Expr::ExprFloat(_) => Ok(Type::TypeFloat),

            // Simple lookup is enough for this
            Expr::ExprIdentifier(i) => Ok(self.env.variables_types.get(i).unwrap().clone()),

            Expr::ExprTupleValue(items) => {
                let mut item_types: Vec<Type> = vec![];
                for item in items {
                    item_types.push(self.calculate(item)?);
                }
                Ok(Type::TypeTuple(item_types))
            }
            Expr::ExprListValue(items) => {
                if items.len() == 0 {
                    return Err("Cant calculate type of list if there is no elements".into());
                }
                let listtype = self.calculate(items.get(0).unwrap())?;
                for item in items {
                    let itemtype = self.calculate(item)?;
                    if listtype != itemtype {
                        return Err(format!(
                            "List type mismatch, expected {:?}, got {:?} in {:?}",
                            listtype, itemtype, expr
                        ));
                    }
                }
                Ok(Type::TypeList(Box::new(listtype)))
            }

            Expr::ExprUnaryOp { op, operand } => {
                calculate_unaryop_type(op, &self.calculate(operand)?)
            }
            Expr::ExprBinOp { left, right, op } => {
                calculate_binaryop_type(op, &self.calculate(left)?, &self.calculate(right)?)
            }

            Expr::ExprListAccess { list, index } => self.calculate_list_access(list, index),

            Expr::ExprNewClassInstance { typename, args }
            | Expr::ExprSpawnActive { typename, args } => {
                let type_origin = self.env.symbol_origins.typenames.get(typename).unwrap();
                let type_definition = self.env.signatures.typenames.get(type_origin);
                if type_definition.is_none() {
                    return sem_err!(
                        "Type definition {} is missing for {:?}",
                        typename, expr
                    );
                }
                let type_definition = type_definition.unwrap();

                if matches!(expr, Expr::ExprNewClassInstance{..}) && type_definition.is_active {
                    return sem_err!("{} is active and must be spawned!", typename);
                } else if matches!(expr, Expr::ExprSpawnActive{..}) && !type_definition.is_active {
                    return sem_err!("Cant spawn passive {}!", typename);
                }
                let constuctor = type_definition
                    .methods
                    .get(typename)
                    .unwrap();
                return self.check_function_call(constuctor, args);
            }
            Expr::ExprFunctionCall { function, args } => {
                let func_origin = self.env.symbol_origins.functions.get(function);
                let func_definition = self.env.signatures.functions.get(func_origin.unwrap());
                if func_definition.is_none() {
                    return sem_err!(
                        "Func definition {} is missing for {:?}",
                        function, expr
                    );
                }
                let func_definition = func_definition.unwrap();
                return self.check_function_call(func_definition, args);
            }

            Expr::ExprMethodCall { object, method, args } => {
                // TODO: implement something for built-in types
                let obj_type = self.calculate(object.as_ref())?;
                match &obj_type {
                    Type::TypeIdentQualified(alias, typename) => {
                        // TODO: checks for type correctness and method correctness
                        let typedef = self.env.signatures.typenames.get(&(*alias, *typename)).unwrap();
                        let method = typedef.methods.get(method).unwrap();
                        return self.check_function_call(method, args);
                    },
                    Type::TypeIdent(t) => panic!("TypeIdent should not be present here!"),
                    Type::TypeMaybe(_) => panic!("Not implemented for maybe yet!"),
                    t => {
                        let method = get_std_method(t, method)?;
                        return self.check_function_call(&method, args);
                    }
                }
            }

            Expr::ExprFieldAccess { object, field } => {
                // TODO: implement something for built-in types
                let obj_type = self.calculate(object.as_ref())?;
                match &obj_type {
                    Type::TypeIdentQualified(alias, typename) => {
                        // TODO: checks for type correctness and method correctness
                        let typedef = self.env.signatures.typenames.get(&(*alias, *typename)).unwrap();
                        let field = typedef.fields.get(field).unwrap();
                        return Ok(field.clone());
                    },
                    Type::TypeIdent(t) => panic!("TypeIdent should not be present here!"),
                    t => Err(format!(
                        "Error at {:?} - type {:?} has no fields",
                        object, obj_type
                    )),
                }
            }
            Expr::ExprOwnMethodCall { .. } | Expr::ExprOwnFieldAccess { .. }
                if self.env.scope.is_none() =>
            {
                Err("Using @ is not allowed in functions".into())
            }
            Expr::ExprOwnMethodCall { method, args } => {
                let type_origin = self.env.scope.as_ref().unwrap();
                let own_definition = self.env.signatures.typenames.get(type_origin).unwrap();
                let method = own_definition.methods.get(method).unwrap();
                return self.check_function_call(method, args);
            }
            Expr::ExprOwnFieldAccess { field } => {
                let type_origin = self.env.scope.as_ref().unwrap();
                let own_definition = self.env.signatures.typenames.get(type_origin);

                let field = own_definition.unwrap().fields.get(field).unwrap();
                return Ok(field.clone());
            }
            Expr::ExprThis => match &self.env.scope {
                None => Err("Using 'this' in the functions is not allowed!".into()),
                Some(o) => {
                    let (module, name) = self.env.scope.as_ref().unwrap().clone();
                    Ok(Type::TypeIdentQualified(module, name))
                },
            },
        }
    }

    fn check_function_call(
        &self,
        function: &FunctionSignature,
        args: &Vec<Expr>,
    ) -> Result<Type, String> {
        if function.args.len() != args.len() {
            return Err(format!(
                "Wrong amount of arguments at {:?}, expected {}",
                args,
                function.args.len()
            ));
        }

        for ((_, arg_type), arg_expr) in function.args.iter().zip(args.iter()) {
            let expr_type = self.calculate(arg_expr)?;
            if arg_type.typename != expr_type {
                return Err(format!(
                    "Wrong type for argument {:?}, got {:?}",
                    arg_type.name, arg_expr
                ));
            }
        }

        Ok(function.rettype.clone())
    }

    fn calculate_list_access(&self, list: &Box<Expr>, index: &Box<Expr>) -> Result<Type, String> {
        let list_type = self.calculate(list.as_ref())?;
        match list_type {
            Type::TypeList(item) => match self.calculate(index)? {
                Type::TypeInt => Ok(item.as_ref().clone()),
                t => Err(format!(
                    "List index must be int, but got {:?} in {:?}",
                    t, index
                )),
            },
            Type::TypeTuple(items) => match index.as_ref() {
                Expr::ExprInt(i) => {
                    let i = *i as usize;
                    let item = items.get(i);
                    if item.is_some() {
                        Ok(item.unwrap().clone())
                    } else {
                        Err(format!("Out of bounds index in {:?}", list.as_ref()))
                    }
                }
                _ => Err(format!("Not int for tuple access in {:?}", list.as_ref())),
            },
            _ => Err(format!(
                "Expected tuple or list for index access, got {:?} in {:?}",
                list_type,
                list.as_ref()
            )),
        }
    }
}
