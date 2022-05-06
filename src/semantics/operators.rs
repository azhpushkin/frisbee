use crate::ast::parsed::{BinaryOp, UnaryOp};
use crate::ast::verified::{RawOperator, VExpr, VExprTyped};
use crate::types::{Type, VerifiedType};

pub fn calculate_unaryop(
    operator: &UnaryOp,
    operand: VExprTyped,
) -> Result<VExprTyped, Box<String>> {
    let exact_operator: RawOperator = match (operator, &operand.expr_type) {
        (UnaryOp::Negate, Type::Int) => RawOperator::UnaryNegateInt,
        (UnaryOp::Negate, Type::Float) => RawOperator::UnaryNegateFloat,
        (UnaryOp::Negate, t) => return Err(Box::new(format!("Can't apply NEGATE to {} type", t))),

        (UnaryOp::Not, Type::Bool) => RawOperator::UnaryNegateBool,
        (UnaryOp::Not, t) => return Err(Box::new(format!("Can't apply NOT to {} type", t))),
    };
    let expr_type = operand.expr_type.clone();

    match operand.expr {
        VExpr::ApplyOp { operator, mut operands } if operator == exact_operator => {
            return Ok(operands.pop().unwrap());
        }
        _ => (),
    }
    Ok(VExprTyped {
        expr: VExpr::ApplyOp { operator: exact_operator, operands: vec![operand] },
        expr_type,
    })
}

pub fn wrap_binary(
    op: RawOperator,
    operands: Vec<VExprTyped>,
    res_type: VerifiedType,
) -> VExprTyped {
    VExprTyped { expr: VExpr::ApplyOp { operator: op, operands }, expr_type: res_type }
}

pub fn calculate_binaryop(
    operator: &BinaryOp,
    left: VExprTyped,
    right: VExprTyped,
) -> Result<VExprTyped, Box<String>> {
    let binaryop_error = format!(
        "Cannot apply {:?} to `{}` and `{}`",
        &operator, &left.expr_type, &right.expr_type
    );

    let ensure_same_types = || {
        (left.expr_type != right.expr_type)
            .then(|| Err(binaryop_error.clone()))
            .unwrap_or(Ok(()))
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
                _ => return Err(Box::new(binaryop_error)),
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

        BinaryOp::And if matches!(left.expr_type, Type::Bool) => {
            ensure_same_types()?;
            (RawOperator::AndBools, Type::Bool)
        }
        BinaryOp::And => return Err(Box::new(binaryop_error)),
        BinaryOp::Or if matches!(left.expr_type, Type::Bool) => {
            ensure_same_types()?;
            (RawOperator::OrBools, Type::Bool)
        }
        BinaryOp::Or => return Err(Box::new(binaryop_error)),
        BinaryOp::IsEqual | BinaryOp::IsNotEqual => {
            unreachable!("Should be handled in ExpressionsVerifier")
        }
        BinaryOp::Elvis => {
            todo!("Elvis not done yet!");
            //     let inner_type = match &left.expr_type {
            //         Type::Maybe(inner_type) => inner_type.as_ref(),
            //         t => {
            //             return Err(format!(
            //                 "Expected Maybe type for elvis operator, got `{}`",
            //                 t
            //             ))
            //         }
            //     };
            //     // TODO: what if the right part of elvis is confirmed via insights?
            //     if &right.expr_type != inner_type {
            //         return Err(format!(
            //             "Right part of elvis must be `{}`, but got `{}`",
            //             inner_type, &right.expr_type
            //         ))
            //     }
            //     let condition = VExprTyped {
            //         expr: VExpr::
            //     }
        }
    };

    Ok(wrap_binary(exact_operator, vec![left, right], result_type))
}
