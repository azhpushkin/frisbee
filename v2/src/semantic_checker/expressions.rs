use std::collections::HashMap;

use super::operators::{calculate_binaryop_type, calculate_unaryop_type};
use crate::ast::*;

pub struct ExprTypeChecker<'a> {
    variables_types: &'a HashMap<String, Type>,
    types_definitions: &'a HashMap<String, ObjectDecl>,
    funcs_definitions: &'a HashMap<String, FunctionDecl>,
}

impl<'a> ExprTypeChecker<'a> {
    pub fn new(
        variables_types: &'a HashMap<String, Type>,
        types_definitions: &'a HashMap<String, ObjectDecl>,
        funcs_definitions: &'a HashMap<String, FunctionDecl>,
    ) -> ExprTypeChecker<'a> {
        ExprTypeChecker { variables_types, types_definitions, funcs_definitions }
    }

    pub fn calculate(&self, expr: &Expr) -> Result<Type, String> {
        match expr {
            // Primitive types, that map to basic types
            Expr::ExprInt(_) => Ok(Type::TypeInt),
            Expr::ExprString(_) => Ok(Type::TypeString),
            Expr::ExprBool(_) => Ok(Type::TypeBool),
            Expr::ExprNil => Ok(Type::TypeNil),
            Expr::ExprFloat(_) => Ok(Type::TypeFloat),

            // Simple lookup is enough for this
            Expr::ExprIdentifier(i) => Ok(self.variables_types.get(i).unwrap().clone()),

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

            Expr::ExprNewClassInstance { typename, args } => {
                let type_definition = self.types_definitions.get(typename);
                if type_definition.is_none() {
                    return Err(format!(
                        "Type definition {} is missing for {:?}",
                        typename, expr
                    ));
                }
                let type_definition = type_definition.unwrap();
                let default_constructor = FunctionDecl {
                    rettype: Type::TypeIdent(typename.clone()),
                    name: typename.clone(),
                    args: type_definition.fields.clone(),
                    statements: vec![],
                };
                let constuctor = type_definition
                    .methods
                    .get(typename)
                    .unwrap_or(&default_constructor);
                return self.check_function_call(constuctor, args);
            }
            Expr::ExprFunctionCall { function, args } => {
                let func_def = self.funcs_definitions.get(function);
                if func_def.is_none() {
                    return Err(format!(
                        "Func definition {} is missing for {:?}",
                        function, expr
                    ));
                }
                let func_def = func_def.unwrap();
                return self.check_function_call(func_def, args);
            }

            Expr::ExprMethodCall { object, method, args } => {
                // TODO: implement something for built-in types
                let obj_type = self.calculate(object.as_ref())?;
                panic!("asd");
            }

            Expr::ExprFieldAccess { .. } => panic!("ExprFieldAccess typecheck not implemented!"),

            Expr::ExprSpawnActive { .. } => panic!("ExprSpawnActive typecheck not implemented!"),
            Expr::ExprThis => panic!("ExprThis typecheck not implemented!"),
        }
    }

    fn check_function_call(
        &self,
        function: &FunctionDecl,
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
