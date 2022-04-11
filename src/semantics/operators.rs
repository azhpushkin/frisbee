use crate::ast::*;

type T = Type;

pub fn calculate_unaryop_type(operator: &UnaryOp, operand: &Type) -> Result<Type, String> {
    return match (operator, operand) {
        (UnaryOp::Negate, Type::Int) => Ok(Type::Int),
        (UnaryOp::Negate, Type::Float) => Ok(Type::Float),
        (UnaryOp::Negate, t) => Err(format!("Cant apply Negate to {:?} type", t)),

        (UnaryOp::Not, Type::Bool) => Ok(Type::Bool),
        (UnaryOp::Not, t) => Err(format!("Cant apply NOT to {:?} type", t)),
    };
}

pub fn calculate_binaryop_type(
    operator: &BinaryOp,
    left: &Type,
    right: &Type,
) -> Result<Type, String> {
    let error_msg = format!("Cant apply {:?} to {:?} and {:?}", operator, left, right);
    let get_res = |is_ok: bool, t: Type| if is_ok { Ok(t) } else { Err(error_msg.clone()) };

    match operator {
        BinaryOp::Plus if left == right => get_res(
            matches!(
                left,
                T::Int | T::Float | T::String | T::List(..)
            ),
            left.clone(),
        ),
        BinaryOp::Plus => Err(error_msg),

        BinaryOp::Minus
        | BinaryOp::Multiply
        | BinaryOp::Divide
        | BinaryOp::Greater
        | BinaryOp::GreaterEqual
        | BinaryOp::Less
        | BinaryOp::LessEqual => get_res(
            left == right && matches!(left, T::Int | T::Float),
            left.clone(),
        ),

        BinaryOp::IsEqual | BinaryOp::IsNotEqual => {
            get_res(are_types_same_or_maybe(left, right), Type::Bool)
        }

        BinaryOp::And | BinaryOp::Or => {
            get_res(left == right && matches!(left, T::Bool), T::Bool)
        }
    }
}

pub fn are_types_same_or_maybe(left: &Type, right: &Type) -> bool {
    match (left, right) {
        (T::Maybe(t1), t2) => t1.as_ref() == t2 || t2 == &T::Nil,
        (t1, T::Maybe(t2)) => t1 == t2.as_ref() || t1 == &T::Nil,
        _ => left == right,
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn check_binary_plus() {
        let t = T::List(Box::new(T::Bool));
        assert_eq!(calculate_binaryop_type(&BinaryOp::Plus, &t, &t).unwrap(), t);

        assert!(
            calculate_binaryop_type(&BinaryOp::Plus, &t, &T::List(Box::new(T::Int)),)
                .is_err()
        );
    }

    #[test]
    pub fn check_binary_equal() {
        assert_eq!(
            calculate_binaryop_type(
                &BinaryOp::IsEqual,
                &T::Maybe(Box::new(T::Bool)),
                &T::Nil
            )
            .unwrap(),
            T::Bool
        );

        assert_eq!(
            calculate_binaryop_type(
                &BinaryOp::IsEqual,
                &T::Bool,
                &T::Maybe(Box::new(T::Bool)),
            )
            .unwrap(),
            T::Bool
        );

        assert!(calculate_binaryop_type(
            &BinaryOp::IsEqual,
            &T::Bool,
            &T::Maybe(Box::new(T::String)),
        )
        .is_err());
    }
}
