use std::collections::HashMap;

use crate::vm::Op;
use crate::ast::*;


// TODO: functions mapping probably needed here
pub fn generate_expr_bytecode(expr: &Expr, locals: &HashMap<&String, usize>) -> Vec<u8> {
    let mut res: Vec<u8> = vec![];
    match expr {
        Expr::ExprBinOp { left, right, op } => {
            res.extend(generate_expr_bytecode(left.as_ref(), locals));
            res.extend(generate_expr_bytecode(right.as_ref(), locals));
            res.push(Op::ADD_INT);  // TODO: use types to understand this, based on TypedExpr
        }
        _ => todo!(),
    }
    return res;
}