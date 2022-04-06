use crate::ast::{BinaryOp, UnaryOp};
use crate::scanner::Token;

pub fn bin_op_from_token(t: &Token) -> BinaryOp {
    match t {
        &Token::Plus => BinaryOp::Plus,
        &Token::Minus => BinaryOp::Minus,
        &Token::Star => BinaryOp::Multiply,
        &Token::Slash => BinaryOp::Divide,
        &Token::Greater => BinaryOp::Greater,
        &Token::GreaterEqual => BinaryOp::GreaterEqual,
        &Token::Less => BinaryOp::Less,
        &Token::LessEqual => BinaryOp::LessEqual,
        &Token::EqualEqual => BinaryOp::IsEqual,
        &Token::BangEqual => BinaryOp::IsNotEqual,
        &Token::And => BinaryOp::And,
        &Token::Or => BinaryOp::Or,
        _ => panic!("Cant convert token {:?} to bin op", t),
    }
}

pub fn unary_op_from_token(t: &Token) -> UnaryOp {
    match t {
        &Token::Minus => UnaryOp::Negate,
        &Token::Not => UnaryOp::Not,
        _ => panic!("Cant convert token {:?} to unary op", t),
    }
}
