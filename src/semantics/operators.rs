use crate::ast::*;
use crate::types::{Type, VerifiedType};

use super::light_ast::{LExpr, LExprTyped, RawOperator};

pub fn calculate_unaryop(operator: &UnaryOp, operand: LExprTyped) -> Result<LExprTyped, String> {
    let exact_operator: RawOperator = match (operator, &operand.expr_type) {
        (UnaryOp::Negate, Type::Int) => RawOperator::UnaryNegateInt,
        (UnaryOp::Negate, Type::Float) => RawOperator::UnaryNegateFloat,
        (UnaryOp::Negate, t) => return Err(format!("Can't apply Negate to {} type", t)),

        (UnaryOp::Not, Type::Bool) => RawOperator::UnaryNegateBool,
        (UnaryOp::Not, t) => return Err(format!("Can't apply NOT to {} type", t)),
    };
    let expr_type = operand.expr_type.clone();
    Ok(LExprTyped {
        expr: LExpr::ApplyOp { operator: exact_operator, operands: vec![operand] },
        expr_type,
    })
}

fn wrap_binary(op: RawOperator, operands: Vec<LExprTyped>, res_type: VerifiedType) -> LExprTyped {
    LExprTyped { expr: LExpr::ApplyOp { operator: op, operands }, expr_type: res_type }
}

pub fn calculate_binaryop(
    operator: &BinaryOp,
    left: LExprTyped,
    right: LExprTyped,
) -> Result<LExprTyped, String> {
    let binaryop_error = format!(
        "Cant apply {:?} to {} and {}",
        &operator, &left.expr_type, &right.expr_type
    );

    let ensure_same_types = || {
        if left.expr_type != right.expr_type {
            Err(binaryop_error.clone())
        } else {
            Ok(())
        }
    };
    let ensure_int_or_float = |int_op: RawOperator, float_op: RawOperator| {
        ensure_same_types()?;
        match left.expr_type {
            Type::Int => Ok((int_op, Type::Int)),
            Type::Float => Ok((float_op, Type::Float)),
            _ => Err(binaryop_error.clone()),
        }
    };
    let ensure_int_or_float_op_only = |int_op: RawOperator, float_op: RawOperator| {
        ensure_int_or_float(int_op, float_op).map(|p| p.0)
    };

    // TODO: greater and less and is_equal for all types?

    let (exact_operator, result_type) = match operator {
        BinaryOp::Plus => {
            ensure_same_types()?;
            match left.expr_type {
                Type::Int => (RawOperator::AddInts, Type::Int),
                Type::Float => (RawOperator::AddFloats, Type::Float),
                Type::String => (RawOperator::AddStrings, Type::String),
                Type::List(_) => todo!("WOW i need to implement this to be fair"),
                _ => return Err(binaryop_error),
            }
        }
        BinaryOp::Minus => ensure_int_or_float(RawOperator::SubInts, RawOperator::SubFloats)?,
        BinaryOp::Multiply => ensure_int_or_float(RawOperator::MulInts, RawOperator::MulFloats)?,
        BinaryOp::Divide => ensure_int_or_float(RawOperator::DivInts, RawOperator::DivFloats)?,

        BinaryOp::Greater => (
            ensure_int_or_float_op_only(RawOperator::GreaterInts, RawOperator::GreaterFloats)?,
            Type::Bool,
        ),
        BinaryOp::Less => (
            ensure_int_or_float_op_only(RawOperator::LessInts, RawOperator::LessFloats)?,
            Type::Bool,
        ),
        BinaryOp::GreaterEqual => {
            let op = ensure_int_or_float_op_only(RawOperator::LessInts, RawOperator::LessFloats)?;
            let inner = wrap_binary(op, vec![left, right], Type::Bool);
            return calculate_unaryop(&UnaryOp::Not, inner);
        }
        BinaryOp::LessEqual => {
            let op =
                ensure_int_or_float_op_only(RawOperator::GreaterInts, RawOperator::GreaterFloats)?;
            let inner = wrap_binary(op, vec![left, right], Type::Bool);
            return calculate_unaryop(&UnaryOp::Not, inner);
        }

        BinaryOp::IsEqual => {
            // TODO: handle maybe here
            ensure_same_types()?;
            let op = match left.expr_type {
                Type::Int => RawOperator::EqualInts,
                Type::Float => RawOperator::EqualFloats,
                Type::Bool => RawOperator::EqualBools,
                Type::String => RawOperator::EqualStrings,
                _ => {
                    return Err(binaryop_error);
                }
            };
            (op, Type::Bool)
        }
        BinaryOp::IsNotEqual => {
            let inner = calculate_binaryop(&BinaryOp::IsEqual, left, right)?;
            return calculate_unaryop(&UnaryOp::Not, inner);
        }

        BinaryOp::And if matches!(left.expr_type, Type::Bool) => {
            ensure_same_types()?;
            (RawOperator::AndBools, Type::Bool)
        }
        BinaryOp::And => return Err(binaryop_error),
        BinaryOp::Or if matches!(left.expr_type, Type::Bool) => {
            ensure_same_types()?;
            (RawOperator::OrBools, Type::Bool)
        }
        BinaryOp::Or => return Err(binaryop_error),
    };

    Ok(wrap_binary(exact_operator, vec![left, right], result_type))
}

// #[cfg(test)]
// pub mod tests {
//     use super::*;

//     #[test]
//     pub fn check_binary_plus() {
//         let t = T::List(Box::new(T::Bool));
//         assert_eq!(calculate_binaryop_type(&BinaryOp::Plus, &t, &t).unwrap(), t);

//         assert!(calculate_binaryop_type(&BinaryOp::Plus, &t, &T::List(Box::new(T::Int)),).is_err());
//     }

//     #[test]
//     pub fn check_binary_equal() {
//         assert_eq!(
//             calculate_binaryop_type(&BinaryOp::IsEqual, &T::Maybe(Box::new(T::Bool)), &T::Nil)
//                 .unwrap(),
//             T::Bool
//         );

//         assert_eq!(
//             calculate_binaryop_type(&BinaryOp::IsEqual, &T::Bool, &T::Maybe(Box::new(T::Bool)),)
//                 .unwrap(),
//             T::Bool
//         );

//         assert!(calculate_binaryop_type(
//             &BinaryOp::IsEqual,
//             &T::Bool,
//             &T::Maybe(Box::new(T::String)),
//         )
//         .is_err());
//     }
// }
