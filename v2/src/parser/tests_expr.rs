use crate::ast::*;

use super::parser_impl::*;
use super::tests_helpers::parse_helper;

fn assert_expr_parses(s: &str, t: Expr) {
    assert_eq!(parse_helper(Parser::parse_expr, s), t);
}

#[test]
fn operator_simple_equality() {
    assert_expr_parses(
        "(nil) == asd",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprNil),
            right: Box::new(Expr::ExprIdentifier(String::from("asd"))),
            op: BinaryOp::IsEqual,
        },
    );
}

#[test]
fn operator_expression() {
    assert_expr_parses(
        "1 + asd",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprInt(1)),
            right: Box::new(Expr::ExprIdentifier(String::from("asd"))),
            op: BinaryOp::Plus,
        },
    );

    assert_expr_parses(
        r#"- "hello""#,
        Expr::ExprUnaryOp {
            operand: Box::new(Expr::ExprString(String::from("hello"))),
            op: UnaryOp::Negate,
        },
    );

    assert_expr_parses(
        "true / 23.2",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprBool(true)),
            right: Box::new(Expr::ExprFloat(23.2)),
            op: BinaryOp::Divide,
        },
    )
}

#[test]
fn expr_minus_minus() {
    assert_expr_parses(
        "-1.0- - 2",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprUnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::ExprFloat(1.0)),
            }),
            right: Box::new(Expr::ExprUnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::ExprInt(2)),
            }),
            op: BinaryOp::Minus,
        },
    )
}

#[test]
fn expr_operator_order() {
    assert_expr_parses(
        "1 + 2 * qw2",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprInt(1)),
            right: Box::new(Expr::ExprBinOp {
                left: Box::new(Expr::ExprInt(2)),
                right: Box::new(Expr::ExprIdentifier(String::from("qw2"))),
                op: BinaryOp::Multiply,
            }),
            op: BinaryOp::Plus,
        },
    )
}

#[test]
fn expr_operator_order_with_grouping() {
    assert_expr_parses(
        "2 * (1 + qw2)",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprInt(2)),
            right: Box::new(Expr::ExprBinOp {
                left: Box::new(Expr::ExprInt(1)),
                right: Box::new(Expr::ExprIdentifier(String::from("qw2"))),
                op: BinaryOp::Plus,
            }),
            op: BinaryOp::Multiply,
        },
    )
}

// TODO: fix this, currently only group is being parsed
#[test]
fn expr_operator_order_with_grouping_and_op_after_group() {
    assert_expr_parses(
        "(1 + qw2) * 2",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprBinOp {
                left: Box::new(Expr::ExprInt(1)),
                right: Box::new(Expr::ExprIdentifier(String::from("qw2"))),
                op: BinaryOp::Plus,
            }),
            right: Box::new(Expr::ExprInt(2)),
            op: BinaryOp::Multiply,
        },
    )
}

#[test]
fn expr_function_call() {
    assert!(false); // todo
}

// TODO: test associativyty
// TODO: test function call
// TODO: test array access
// TODO: test array

// statements
// TODO: test array assignment
// TODO
