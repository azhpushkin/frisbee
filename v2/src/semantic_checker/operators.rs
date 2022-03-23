use crate::ast::*;

type T = Type;

pub fn calculate_unaryop_type(operator: &UnaryOp, operand: &Type) -> Result<Type, String> {
    return match (operator, operand) {
        (UnaryOp::Negate, Type::TypeInt) => Ok(Type::TypeInt),
        (UnaryOp::Negate, Type::TypeFloat) => Ok(Type::TypeFloat),
        (UnaryOp::Negate, t) => Err(format!("Cant apply Negate to {:?} type", t)),

        (UnaryOp::Not, Type::TypeBool) => Ok(Type::TypeBool),
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
                T::TypeInt | T::TypeFloat | T::TypeString | T::TypeList(..)
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
            left == right && matches!(left, T::TypeInt | T::TypeFloat),
            left.clone(),
        ),

        BinaryOp::IsEqual | BinaryOp::IsNotEqual => match (left, right) {
            (T::TypeMaybe(t1), t2) => get_res(t1.as_ref() == t2 || t2 == &T::TypeNil, T::TypeBool),
            (t1, T::TypeMaybe(t2)) => get_res(t1 == t2.as_ref() || t1 == &T::TypeNil, T::TypeBool),
            _ => get_res(left == right, T::TypeBool),
        },

        BinaryOp::And | BinaryOp::Or => {
            get_res(left == right && matches!(left, T::TypeBool), T::TypeBool)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn check_binary_plus() {
        let t = T::TypeList(Box::new(T::TypeBool));
        assert_eq!(calculate_binaryop_type(&BinaryOp::Plus, &t, &t).unwrap(), t);

        assert!(
            calculate_binaryop_type(&BinaryOp::Plus, &t, &T::TypeList(Box::new(T::TypeInt)),)
                .is_err()
        );
    }

    #[test]
    pub fn check_binary_equal() {
        assert_eq!(
            calculate_binaryop_type(
                &BinaryOp::IsEqual,
                &T::TypeMaybe(Box::new(T::TypeBool)),
                &T::TypeNil
            )
            .unwrap(),
            T::TypeBool
        );

        assert_eq!(
            calculate_binaryop_type(
                &BinaryOp::IsEqual,
                &T::TypeBool,
                &T::TypeMaybe(Box::new(T::TypeBool)),
            )
            .unwrap(),
            T::TypeBool
        );

        assert!(calculate_binaryop_type(
            &BinaryOp::IsEqual,
            &T::TypeBool,
            &T::TypeMaybe(Box::new(T::TypeString)),
        )
        .is_err());
    }
}
