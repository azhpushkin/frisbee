use crate::vm::Op;
use crate::ast::*;


pub fn generate_for_expr(expr: &Expr) -> Vec<u8> {
    let mut res: Vec<u8> = vec![];
    match expr {
        Expr::ExprBinOp { left, right, op } => {
            res.extend(generate_for_expr(left.as_ref()));
            res.extend(generate_for_expr(right.as_ref()));
            res.push(Op::ADD_INT);

        }
        _ => todo!(),
    }
    return res;
}