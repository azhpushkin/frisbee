use std::collections::HashMap;

use super::operators::{calculate_binaryop_type, calculate_unaryop_type};
use crate::ast::*;

pub fn calculate_expr_type(
    expr: &Expr,
    variables: &HashMap<String, Type>,
    types: &HashMap<String, ObjectDecl>,
) -> Result<Type, String> {
    match expr {
        // Primitive types, that map to basic types
        Expr::ExprInt(_) => Ok(Type::TypeInt),
        Expr::ExprString(_) => Ok(Type::TypeString),
        Expr::ExprBool(_) => Ok(Type::TypeBool),
        Expr::ExprNil => Ok(Type::TypeNil),
        Expr::ExprFloat(_) => Ok(Type::TypeFloat),

        // Simple lookup is enough for this
        Expr::ExprIdentifier(i) => Ok(variables.get(i).unwrap().clone()),

        Expr::ExprTupleValue(items) => {
            let mut item_types: Vec<Type> = vec![];
            for item in items {
                item_types.push(calculate_expr_type(item, variables, types)?);
            }
            Ok(Type::TypeTuple(item_types))
        }

        Expr::ExprUnaryOp { op, operand } => {
            calculate_unaryop_type(op, &calculate_expr_type(operand, variables, types)?)
        }
        Expr::ExprBinOp { left, right, op } => calculate_binaryop_type(
            op,
            &calculate_expr_type(left, variables, types)?,
            &calculate_expr_type(right, variables, types)?,
        ),
        Expr::ExprListAccess { list, index } => {
            let list_type = calculate_expr_type(list.as_ref(), variables, types)?;
            match list_type {
                Type::TypeList(item) => match calculate_expr_type(index, variables, types)? {
                    Type::TypeInt => Ok(item.as_ref().clone()),
                    t => Err(format!(
                        "List index must be int, but got {:?} in {:?}",
                        t, expr
                    )),
                },
                Type::TypeTuple(items) => match index.as_ref() {
                    Expr::ExprInt(i) => {
                        let i = *i as usize;
                        let item = items.get(i);
                        if item.is_some() {
                            Ok(item.unwrap().clone())
                        } else {
                            Err(format!("Out of bounds index in {:?}", expr))
                        }
                    }
                    _ => Err(format!("Not int for tuple access in {:?}", expr)),
                },
                _ => Err(format!(
                    "Expected tuple or list for index access, got {:?} in {:?}",
                    list_type, expr
                )),
            }
        }
        Expr::ExprListValue(_) => panic!("ExprListValue typecheck not implemented!"),

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
