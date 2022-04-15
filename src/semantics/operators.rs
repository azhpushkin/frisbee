use crate::ast::*;

use super::light_ast::{LExpr, LExprTyped, RawOperator};

type T = Type;

pub fn calculate_unaryop(operator: &UnaryOp, operand: LExprTyped) -> LExprTyped {
    let exact_operator: RawOperator = match (operator, &operand.expr_type) {
        (UnaryOp::Negate, Type::Int) => RawOperator::UnaryNegateInt,
        (UnaryOp::Negate, Type::Float) => RawOperator::UnaryNegateFloat,
        (UnaryOp::Negate, t) => panic!("Cant apply Negate to {:?} type", t),

        (UnaryOp::Not, Type::Bool) => RawOperator::UnaryNegateBool,
        (UnaryOp::Not, t) => panic!("Cant apply NOT to {:?} type", t),
    };
    let expr_type = operand.expr_type.clone();
    LExprTyped {
        expr: LExpr::ApplyOp { operator: exact_operator, operands: vec![operand] },
        expr_type
    }
}

fn wrap_binary(op: RawOperator, operands: Vec<LExprTyped>, res_type: Type) -> LExprTyped {
    LExprTyped {
        expr: LExpr::ApplyOp { operator: op, operands },
        expr_type: res_type,
    }
}

pub fn calculate_binaryop(
    operator: &BinaryOp,
    left: LExprTyped,
    right: LExprTyped,
) -> LExprTyped {
    let raise_error = || panic!("Cant apply {:?} to {:?} and {:?}", &operator, &left, &right);

    let ensure_same_types = || {
        if left.expr_type != right.expr_type {
            raise_error();
        }
    };
    let ensure_int_or_float = |int_op: RawOperator, float_op: RawOperator| {
        ensure_same_types();
        match left.expr_type {
            Type::Int => (int_op, Type::Int),
            Type::Float => (float_op, Type::Float),
            _ => raise_error()
        }
    };
    let ensure_int_or_float_op_only = |int_op: RawOperator, float_op: RawOperator| {
        ensure_int_or_float(int_op, float_op).0
    };
    // TODO: greater and less and is_equal for all types?

    let (exact_operator, result_type) = match operator {
        BinaryOp::Plus => {
            ensure_same_types();
            match left.expr_type {
                Type::Int => (RawOperator::AddInts, Type::Int),
                Type::Float => (RawOperator::AddFloats, Type::Float),
                Type::String => (RawOperator::AddFloats, Type::Float),
                Type::List(_) => panic!("WOW i need to implement this to be fair"),
                _ => raise_error()
            }
        }
        BinaryOp::Minus => ensure_int_or_float(RawOperator::SubInts, RawOperator::SubFloats),
        BinaryOp::Multiply => ensure_int_or_float(RawOperator::MulInts, RawOperator::MulFloats),
        BinaryOp::Divide => ensure_int_or_float(RawOperator::DivInts, RawOperator::DivFloats),
        
        BinaryOp::Greater => (
            ensure_int_or_float_op_only(RawOperator::GreaterInts, RawOperator::GreaterFloats),
            Type::Bool
        ),
        BinaryOp::Less => (
            ensure_int_or_float_op_only(RawOperator::LessInts, RawOperator::LessFloats),
            Type::Bool
        ),
        BinaryOp::GreaterEqual => {
            let op = ensure_int_or_float_op_only(RawOperator::LessInts, RawOperator::LessFloats);
            let inner = wrap_binary(op, vec![left, right], Type::Bool);
            return calculate_unaryop(&UnaryOp::Not, inner);
        },
        BinaryOp::LessEqual => {
            let op = ensure_int_or_float_op_only(RawOperator::GreaterInts, RawOperator::GreaterFloats);
            let inner = wrap_binary(op, vec![left, right], Type::Bool);
            return calculate_unaryop(&UnaryOp::Not, inner);
        },

        _ => todo!(),
    };

        // for both bool and numbers
        // | BinaryOp::Greater

        // | BinaryOp::Less
        // |  => get_res(
        //     left == right && matches!(left, T::Int | T::Float),
        //     left.clone(),
        // ),

        // BinaryOp::GreaterEqual => todo!(),  // Greater and equal
        // BinaryOp::LessEqual => todo!(),  //  Less AND equal

        // // for all types, and also special case for maybe
        // BinaryOp::IsEqual | BinaryOp::IsNotEqual => {
        //     get_res(are_types_same_or_maybe(left, right), Type::Bool)
        // }

        // // Only for bool
        // BinaryOp::And | BinaryOp::Or => get_res(left == right && matches!(left, T::Bool), T::Bool),
    
    wrap_binary(exact_operator, vec![left, right], result_type)
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
