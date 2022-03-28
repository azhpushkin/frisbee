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

    pub fn calculate(&mut self, expr: &Expr) -> Result<Type, String> {
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

            Expr::ExprUnaryOp { op, operand } => {
                calculate_unaryop_type(op, &self.calculate(operand)?)
            }
            Expr::ExprBinOp { left, right, op } => {
                calculate_binaryop_type(op, &self.calculate(left)?, &self.calculate(right)?)
            }
            Expr::ExprListAccess { list, index } => self.calculate_list_access(list, index),
            Expr::ExprListValue(items) => {
                if items.len() == 0 {
                    return Err("Cant calculate type of list if there is no elements".into());
                }
                let listtype = self.calculate(items.get(0).unwrap())?;
                Err("".into()) // TODO
            }

            Expr::ExprMethodCall { .. } => panic!("ExprMethodCall typecheck not implemented!"),
            Expr::ExprFunctionCall { .. } => panic!("ExprFunctionCall typecheck not implemented!"),
            Expr::ExprFieldAccess { .. } => panic!("ExprFieldAccess typecheck not implemented!"),

            Expr::ExprNewClassInstance { .. } => {
                panic!("ExprNewClassInstance typecheck not implemented!")
            }
            Expr::ExprSpawnActive { .. } => panic!("ExprSpawnActive typecheck not implemented!"),
            Expr::ExprThis => panic!("ExprThis typecheck not implemented!"),
        }
    }

    fn calculate_list_access(
        &mut self,
        list: &Box<Expr>,
        index: &Box<Expr>,
    ) -> Result<Type, String> {
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
