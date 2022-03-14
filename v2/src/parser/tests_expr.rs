use crate::ast::*;

use super::parser_impl::*;
use super::tests_helpers::*;

fn assert_expr_parses(s: &str, t: Expr) {
    assert_eq!(parse_and_unwrap(Parser::parse_expr, s), t);
}

fn assert_expr_invalid(s: &str) {
    assert_parsing_fails(Parser::parse_expr, s);
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
fn expr_simple_groups() {
    assert_expr_parses("(1)", Expr::ExprInt(1));
    assert_expr_parses(
        "(-(-3))",
        Expr::ExprUnaryOp {
            op: UnaryOp::Negate,
            operand: Box::new(Expr::ExprUnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::ExprInt(3)),
            }),
        },
    );
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
    );

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
    );
}

#[test]
fn expr_tuple() {
    assert_expr_parses(
        "(1, 2.0, ad)",
        Expr::ExprTupleValue(vec![
            Expr::ExprInt(1),
            Expr::ExprFloat(2.0),
            Expr::ExprIdentifier(String::from("ad")),
        ]),
    );

    assert_expr_parses(
        "((1, 2), (3, 4))",
        Expr::ExprTupleValue(vec![
            Expr::ExprTupleValue(vec![Expr::ExprInt(1), Expr::ExprInt(2)]),
            Expr::ExprTupleValue(vec![Expr::ExprInt(3), Expr::ExprInt(4)]),
        ]),
    );
}

#[test]
fn expr_group_and_tuple_mixed() {
    assert_expr_parses(
        "((1, 2) + (3, 4))",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprTupleValue(vec![
                Expr::ExprInt(1),
                Expr::ExprInt(2),
            ])),
            right: Box::new(Expr::ExprTupleValue(vec![
                Expr::ExprInt(3),
                Expr::ExprInt(4),
            ])),
            op: BinaryOp::Plus,
        },
    );
}

#[test]
fn expr_bad_parenthesis_usage() {
    assert_expr_invalid("()");
    assert_expr_invalid("(, )");
    assert_expr_invalid("(21 +2");

    // Tuple of single value is not allowed
    assert_expr_invalid("(2, )");
}

#[test]
fn expr_list_value() {
    assert_expr_parses("[]", Expr::ExprListValue(vec![]));

    assert_expr_parses(
        "[1, ooi]",
        Expr::ExprListValue(vec![
            Expr::ExprInt(1),
            Expr::ExprIdentifier(String::from("ooi")),
        ]),
    );

    // trailing comma is allowed
    assert_expr_parses(
        "[nil, 2.0,]",
        Expr::ExprListValue(vec![Expr::ExprNil, Expr::ExprFloat(2.0)]),
    );

    assert_expr_invalid("[, ]");
}

#[test]
fn expr_list_access() {
    assert_expr_parses(
        "asd[2]",
        Expr::ExprListAccess {
            list: Box::new(Expr::ExprIdentifier(String::from("asd"))),
            index: Box::new(Expr::ExprInt(2)),
        },
    );
}

#[test]
fn expr_list_access_chained() {
    assert_expr_parses(
        "asd[2][0]",
        Expr::ExprListAccess {
            list: Box::new(Expr::ExprListAccess {
                list: Box::new(Expr::ExprIdentifier(String::from("asd"))),
                index: Box::new(Expr::ExprInt(2)),
            }),
            index: Box::new(Expr::ExprInt(0)),
        },
    );
}

#[test]
fn expr_field_access() {
    assert_expr_parses(
        "(1).qwe",
        Expr::ExprFieldAccess { object: Box::new(Expr::ExprInt(1)), field: String::from("qwe") },
    );

    assert_expr_invalid("obj.(field)");
}

#[test]
fn expr_field_access_chained_with_method_call() {
    assert_expr_parses(
        "obj.field.method()",
        Expr::ExprMethodCall {
            object: Box::new(Expr::ExprFieldAccess {
                object: Box::new(Expr::ExprIdentifier(String::from("obj"))),
                field: String::from("field"),
            }),
            method: String::from("method"),
            args: vec![],
        },
    );

    assert_expr_invalid("obj.field.(method())");
    assert_expr_invalid("obj.(method())");
    assert_expr_invalid("obj.(method)()");
}

#[test]
fn expr_method_call() {
    assert_expr_parses(
        "1.qwe()",
        Expr::ExprMethodCall {
            object: Box::new(Expr::ExprInt(1)),
            method: String::from("qwe"),
            args: vec![],
        },
    );
}

#[test]
fn expr_method_call_with_args() {
    assert_expr_parses(
        "asd.qwe(1, true, this)",
        Expr::ExprMethodCall {
            object: Box::new(Expr::ExprIdentifier(String::from("asd"))),
            method: String::from("qwe"),
            args: vec![Expr::ExprInt(1), Expr::ExprBool(true), Expr::ExprThis],
        },
    );
}

#[test]
fn expr_method_call_chained() {
    assert_expr_parses(
        "(1, 2).qwe().asd()",
        Expr::ExprMethodCall {
            object: Box::new(Expr::ExprMethodCall {
                object: Box::new(Expr::ExprTupleValue(vec![
                    Expr::ExprInt(1),
                    Expr::ExprInt(2),
                ])),
                method: String::from("qwe"),
                args: vec![],
            }),
            method: String::from("asd"),
            args: vec![],
        },
    );
}

#[test]
fn expr_call_method_on_list_literal() {
    let expected = Expr::ExprMethodCall {
        object: Box::new(Expr::ExprListValue(vec![
            Expr::ExprIdentifier(String::from("asd")),
            Expr::ExprInt(2),
        ])),
        method: String::from("qwe"),
        args: vec![Expr::ExprThis],
    };

    assert_expr_parses("[asd, 2].qwe(this)", expected.clone());
    assert_expr_parses("([asd, 2]).qwe(this)", expected.clone());
}

#[test]
fn expr_call_method_on_list_access() {
    assert_expr_parses(
        "[asd, 2][0].qwe(true)",
        Expr::ExprMethodCall {
            object: Box::new(Expr::ExprListAccess {
                list: Box::new(Expr::ExprListValue(vec![
                    Expr::ExprIdentifier(String::from("asd")),
                    Expr::ExprInt(2),
                ])),
                index: Box::new(Expr::ExprInt(0)),
            }),
            method: String::from("qwe"),
            args: vec![Expr::ExprThis],
        },
    );
}
// TODO: test associativyty (5 / 2 / 3 and 5 - 3 - 2)
// TODO: test function call
// TODO: test array access
// TODO: test array

// statements
// TODO: test array assignment
// TODO
