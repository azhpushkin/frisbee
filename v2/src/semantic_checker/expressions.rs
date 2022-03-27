use std::collections::HashMap;

use crate::ast::*;

pub fn calculate_expression_type(
    expr: &Expr,
    variables_types: HashMap<String, Type>,
    types_definitions: HashMap<String, ObjectDecl>,
) -> Type {
    match expr {
        // Primitive types, that map to basic types
        Expr::ExprInt(_) => Type::TypeInt,
        Expr::ExprString(_) => Type::TypeString,
        Expr::ExprBool(_) => Type::TypeBool,
        Expr::ExprNil => Type::TypeNil,
        Expr::ExprFloat(_) => Type::TypeFloat,

        // Simple lookup is enough for this
        Expr::ExprIdentifier(i) => variables_types.get(i).unwrap().clone(),

        Expr::ExprUnaryOp { op, operand } => panic!("ExprUnaryOp typecheck not implemented!"),
        Expr::ExprBinOp { left, right, op } => panic!("ExprBinOp typecheck not implemented!"),
        Expr::ExprListAccess { list, index } => panic!("ExprListAccess typecheck not implemented!"),
        Expr::ExprListValue(_) => panic!("ExprListValue typecheck not implemented!"),
        Expr::ExprTupleValue(_) => panic!("ExprTupleValue typecheck not implemented!"),
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
