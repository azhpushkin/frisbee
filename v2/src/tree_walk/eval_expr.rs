use super::state::State;
use crate::ast::*;
use crate::utils::extract_result_if_ok;

type FinalExpr = Result<Expr, String>;

// TODO: how to compare Maybe type with nil?
pub fn eval_binary_expr(op: &BinaryOp, left: &Expr, right: &Expr) -> FinalExpr {
    let expr: Expr = match (op, left, right) {
        (BinaryOp::And, Expr::ExprBool(b1), Expr::ExprBool(b2)) => Expr::ExprBool(*b1 && *b2),
        (BinaryOp::Or, Expr::ExprBool(b1), Expr::ExprBool(b2)) => Expr::ExprBool(*b1 || *b2),

        (BinaryOp::IsEqual, Expr::ExprBool(b1), Expr::ExprBool(b2)) => Expr::ExprBool(b1 == b2),
        (BinaryOp::IsEqual, Expr::ExprFloat(f1), Expr::ExprFloat(f2)) => Expr::ExprBool(f1 == f2),
        (BinaryOp::IsEqual, Expr::ExprInt(f1), Expr::ExprInt(f2)) => Expr::ExprBool(f1 == f2),
        (BinaryOp::IsEqual, Expr::ExprString(f1), Expr::ExprString(f2)) => Expr::ExprBool(f1 == f2),
        (BinaryOp::IsEqual, Expr::ExprNil, Expr::ExprNil) => Expr::ExprBool(true),
        (BinaryOp::IsEqual, _, Expr::ExprNil) => Expr::ExprBool(false),
        (BinaryOp::IsEqual, Expr::ExprNil, _) => Expr::ExprBool(false),

        (BinaryOp::IsNotEqual, Expr::ExprBool(b1), Expr::ExprBool(b2)) => {
            Expr::ExprBool(*b1 != *b2)
        }

        (_, _, _) => {
            return Err(format!(
                "Cant apply {:?} to L: {:?} and R: {:?}",
                op,
                left.as_ref(),
                right.as_ref()
            ))
        }
    };

    Ok(expr)
}

pub fn eval_expr(expr: &Expr, state: &State) -> FinalExpr {
    let result: Expr = match expr {
        &Expr::ExprInt(_) => expr.clone(),
        &Expr::ExprFloat(_) => expr.clone(),
        &Expr::ExprString(_) => expr.clone(),
        &Expr::ExprBool(_) => expr.clone(),
        &Expr::ExprNil => return Ok(Expr::ExprNil),

        &Expr::ExprUnaryOp { op, operand } => {
            let operand_evaluated = extract_result_if_ok!(eval_expr(operand.as_ref(), state));
            match (&op, &operand_evaluated) {
                (UnaryOp::Not, Expr::ExprBool(b)) => Expr::ExprBool(!b),
                (UnaryOp::Negate, Expr::ExprFloat(f)) => Expr::ExprFloat(-f),
                (UnaryOp::Negate, Expr::ExprInt(i)) => Expr::ExprInt(-i),
                (_, _) => return Err(format!("Cant apply {:?} to  {:?} ", op, expr)),
            }
        }
        &Expr::ExprBinOp { left, right, op } => {
            let leftexpr = extract_result_if_ok!(eval_expr(left.as_ref(), state));
            let rightexpr = extract_result_if_ok!(eval_expr(right.as_ref(), state));
            extract_result_if_ok!(eval_binary_expr(&op, &leftexpr, &rightexpr))
        } // &Expr::ExprListAccess{} => return 123;
          // &Expr::ExprListValue{} => return 123;
          // &Expr::ExprTupleValue{} => return 123;
          // &Expr::ExprMethodCall{} => return 123;
          // &Expr::ExprFieldAccess{} => return 123;

          // &Expr::ExprIdentifier{} => return 123;
          // &Expr::ExprNewClassInstance{} => return 123;
          // &Expr::ExprSpawnActive{} => return 123;
          // &Expr::ExprThis{} => return 123;
    };

    Ok(result)
}
