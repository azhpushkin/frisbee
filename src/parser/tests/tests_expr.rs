use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

fn assert_expr_parses(s: &str, t: Expr) {
    let with_pos = ExprWithPos { expr: t, pos_first: 0, pos_last: s.len() - 1 };
    assert_eq!(parse_and_unwrap(Parser::parse_expr, s), with_pos);
}

fn assert_expr_parses_padded(s: &str, t: Expr, first: usize, last: usize) {
    let with_pos = ExprWithPos { expr: t, pos_first: first, pos_last: last };
    assert_eq!(parse_and_unwrap(Parser::parse_expr, s), with_pos);
}

fn assert_expr_invalid(s: &str) {
    assert_parsing_fails(Parser::parse_expr, s);
}

fn expr(e: Expr, f: usize, l: usize) -> Box<ExprWithPos> {
    Box::new(expr_raw(e, f, l))
}

fn expr_raw(e: Expr, f: usize, l: usize) -> ExprWithPos {
    ExprWithPos { expr: e, pos_first: f, pos_last: l }
}

#[test]
fn string_single_and_double_quotes() {
    assert_expr_parses_padded(r#" 'asd' "#, Expr::String("asd".into()), 1, 5);
    assert_expr_parses_padded(r#" "asd" "#, Expr::String("asd".into()), 1, 5);
}

#[test]
fn operator_simple_equality() {
    assert_expr_parses(
        "(nil) == asd",
        Expr::BinOp {
            left: expr(Expr::Nil, 1, 3),
            right: expr(Expr::Identifier("asd".into()), 9, 11),
            op: BinaryOp::IsEqual,
        },
    );
}

#[test]
fn operator_expression() {
    assert_expr_parses(
        "1 + asd",
        Expr::BinOp {
            left: expr(Expr::Int(1), 0, 0),
            right: expr(Expr::Identifier("asd".into()), 4, 6),
            op: BinaryOp::Plus,
        },
    );

    assert_expr_parses(
        r#"- "hello""#,
        Expr::UnaryOp { operand: expr(Expr::String("hello".into()), 2, 8), op: UnaryOp::Negate },
    );

    assert_expr_parses(
        "true / 23.2",
        Expr::BinOp {
            left: expr(Expr::Bool(true), 0, 3),
            right: expr(Expr::Float(23.2), 7, 10),
            op: BinaryOp::Divide,
        },
    )
}

#[test]
fn expr_minus_minus() {
    assert_expr_parses(
        "-1.0- - 2",
        Expr::BinOp {
            left: expr(
                Expr::UnaryOp { op: UnaryOp::Negate, operand: expr(Expr::Float(1.0), 1, 3) },
                0,
                3,
            ),
            right: expr(
                Expr::UnaryOp { op: UnaryOp::Negate, operand: expr(Expr::Int(2), 8, 8) },
                6,
                8,
            ),
            op: BinaryOp::Minus,
        },
    )
}

#[test]
fn expr_operator_order() {
    assert_expr_parses(
        "1 + 2 * qw2",
        Expr::BinOp {
            left: expr(Expr::Int(1), 0, 0),
            right: expr(
                Expr::BinOp {
                    left: expr(Expr::Int(2), 4, 4),
                    right: expr(Expr::Identifier("qw2".into()), 8, 10),
                    op: BinaryOp::Multiply,
                },
                4,
                10,
            ),
            op: BinaryOp::Plus,
        },
    )
}

#[test]
fn expr_simple_groups() {
    assert_expr_parses_padded("(1)", Expr::Int(1), 1, 1);
    assert_expr_parses_padded(
        "(-(-3))",
        Expr::UnaryOp {
            op: UnaryOp::Negate,
            operand: expr(
                Expr::UnaryOp { op: UnaryOp::Negate, operand: expr(Expr::Int(3), 4, 4) },
                3,
                4,
            ),
        },
        1,
        5,
    );
}

#[test]
fn expr_operator_order_with_grouping() {
    assert_expr_parses(
        "2 * (1 + qw2)",
        Expr::BinOp {
            left: expr(Expr::Int(2), 0, 0),
            right: expr(
                Expr::BinOp {
                    left: expr(Expr::Int(1), 5, 5),
                    right: expr(Expr::Identifier("qw2".into()), 9, 11),
                    op: BinaryOp::Plus,
                },
                5,
                11,
            ),
            op: BinaryOp::Multiply,
        },
    );

    assert_expr_parses(
        "(1 + qw2) * 2",
        Expr::BinOp {
            left: expr(
                Expr::BinOp {
                    left: expr(Expr::Int(1), 1, 1),
                    right: expr(Expr::Identifier("qw2".into()), 5, 7),
                    op: BinaryOp::Plus,
                },
                1,
                7,
            ),
            right: expr(Expr::Int(2), 12, 12),
            op: BinaryOp::Multiply,
        },
    );
}

#[test]
fn expr_tuple() {
    assert_expr_parses(
        "(1, 2.0, ad)",
        Expr::TupleValue(vec![
            expr_raw(Expr::Int(1), 1, 1),
            expr_raw(Expr::Float(2.0), 4, 6),
            expr_raw(Expr::Identifier("ad".into()), 9, 10),
        ]),
    );

    assert_expr_parses(
        "((1, 2), (3, 4))",
        Expr::TupleValue(vec![
            expr_raw(
                Expr::TupleValue(vec![
                    expr_raw(Expr::Int(1), 2, 2),
                    expr_raw(Expr::Int(2), 5, 5),
                ]),
                1,
                6,
            ),
            expr_raw(
                Expr::TupleValue(vec![
                    expr_raw(Expr::Int(3), 10, 10),
                    expr_raw(Expr::Int(4), 13, 13),
                ]),
                9,
                14,
            ),
        ]),
    );

    // single-element tuple is simplified to just an element
    assert_expr_parses_padded("(1, )", Expr::Int(1), 1, 1);
}

#[test]
fn expr_group_and_tuple_mixed() {
    assert_expr_parses_padded(
        "((1, 2) + (3, 4))",
        Expr::BinOp {
            left: expr(
                Expr::TupleValue(vec![
                    expr_raw(Expr::Int(1), 2, 2),
                    expr_raw(Expr::Int(2), 5, 5),
                ]),
                1,
                6,
            ),
            right: expr(
                Expr::TupleValue(vec![
                    expr_raw(Expr::Int(3), 11, 11),
                    expr_raw(Expr::Int(4), 14, 14),
                ]),
                10,
                15,
            ),
            op: BinaryOp::Plus,
        },
        1,
        15,
    );
}

#[test]
fn expr_bad_parenthesis_usage() {
    assert_expr_invalid("()");
    assert_expr_invalid("(, )");
    assert_expr_invalid("(21 +2");
}

#[test]
fn expr_list_value() {
    assert_expr_parses("[]", Expr::ListValue(vec![]));

    assert_expr_parses(
        "[1, new]",
        Expr::ListValue(vec![
            expr_raw(Expr::Int(1), 1, 1),
            expr_raw(Expr::Identifier("new".into()), 4, 6),
        ]),
    );

    // trailing comma is allowed
    assert_expr_parses(
        "[nil, 2.0,]",
        Expr::ListValue(vec![
            expr_raw(Expr::Nil, 1, 3),
            expr_raw(Expr::Float(2.0), 6, 8),
        ]),
    );

    assert_expr_invalid("[, ]");
}

#[test]
fn expr_list_access() {
    assert_expr_parses(
        "asd[2]",
        Expr::ListAccess {
            list: expr(Expr::Identifier("asd".into()), 0, 2),
            index: expr(Expr::Int(2), 4, 4),
        },
    );
}

#[test]
fn expr_list_access_chained() {
    assert_expr_parses(
        "asd[2][0]",
        Expr::ListAccess {
            list: expr(
                Expr::ListAccess {
                    list: expr(Expr::Identifier("asd".into()), 0, 2),
                    index: expr(Expr::Int(2), 4, 4),
                },
                0,
                5,
            ),
            index: expr(Expr::Int(0), 7, 7),
        },
    );
}

#[test]
fn expr_field_access() {
    assert_expr_parses(
        "(1).qwe",
        Expr::FieldAccess { object: expr(Expr::Int(1), 1, 1), field: "qwe".into() },
    );

    assert_expr_invalid("obj.(field)");
}

#[test]
fn expr_own_field_access() {
    assert_expr_parses("@qwe", Expr::OwnFieldAccess { field: "qwe".into() });
}

#[test]
fn expr_func() {
    assert_expr_parses(
        "(function) (1, )",
        Expr::FunctionCall {
            function: "function".into(),
            args: vec![expr_raw(Expr::Int(1), 12, 12)],
        },
    );

    assert_expr_invalid("function.()");
    assert_expr_invalid("function()()");
}

#[test]
fn expr_field_access_chained_with_method_call() {
    assert_expr_parses(
        "obj.field.method()",
        Expr::MethodCall {
            object: expr(
                Expr::FieldAccess {
                    object: expr(Expr::Identifier("obj".into()), 0, 2),
                    field: "field".into(),
                },
                0,
                8,
            ),
            method: "method".into(),
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
        Expr::MethodCall { object: expr(Expr::Int(1), 0, 0), method: "qwe".into(), args: vec![] },
    );
}

#[test]
fn expr_own_method_call() {
    assert_expr_parses_padded(
        "(@qwe())",
        Expr::OwnMethodCall { method: "qwe".into(), args: vec![] },
        1,
        6,
    );
}

#[test]
fn expr_method_call_with_args() {
    assert_expr_parses(
        "asd.qwe(1, )",
        Expr::MethodCall {
            object: expr(Expr::Identifier("asd".into()), 0, 2),
            method: "qwe".into(),
            args: vec![expr_raw(Expr::Int(1), 8, 8)],
        },
    );

    assert_expr_parses(
        "asd.qwe(1, true, this)",
        Expr::MethodCall {
            object: expr(Expr::Identifier("asd".into()), 0, 2),
            method: "qwe".into(),
            args: vec![
                expr_raw(Expr::Int(1), 8, 8),
                expr_raw(Expr::Bool(true), 11, 14),
                expr_raw(Expr::This, 17, 20),
            ],
        },
    );
}

#[test]
fn expr_own_method_call_and_field_access() {
    assert_expr_parses(
        "@qwe().next(@field)",
        Expr::MethodCall {
            object: expr(
                Expr::OwnMethodCall { method: "qwe".into(), args: vec![] },
                0,
                5,
            ),
            method: "next".into(),
            args: vec![expr_raw(Expr::OwnFieldAccess { field: "field".into() }, 12, 17)],
        },
    );
}

#[test]
fn expr_method_call_chained() {
    assert_expr_parses(
        "(1, 2).qwe().asd()",
        Expr::MethodCall {
            object: expr(
                Expr::MethodCall {
                    object: expr(
                        Expr::TupleValue(vec![
                            expr_raw(Expr::Int(1), 1, 1),
                            expr_raw(Expr::Int(2), 4, 4),
                        ]),
                        0,
                        5,
                    ),
                    method: "qwe".into(),
                    args: vec![],
                },
                0,
                11,
            ),
            method: "asd".into(),
            args: vec![],
        },
    );
}

#[test]
fn expr_call_method_on_list_literal() {
    let expected = Expr::MethodCall {
        object: expr(
            Expr::ListValue(vec![
                expr_raw(Expr::Identifier("asd".into()), 2, 4),
                expr_raw(Expr::Int(2), 7, 7),
            ]),
            1,
            8,
        ),
        method: "qwe".into(),
        args: vec![expr_raw(Expr::This, 15, 18)],
    };

    assert_expr_parses("([asd, 2]).qwe(this)", expected);
}

#[test]
fn expr_call_method_on_list_access() {
    assert_expr_parses(
        "[asd, 2][0].qwe(this)",
        Expr::MethodCall {
            object: expr(
                Expr::ListAccess {
                    list: expr(
                        Expr::ListValue(vec![
                            expr_raw(Expr::Identifier("asd".into()), 1, 3),
                            expr_raw(Expr::Int(2), 6, 6),
                        ]),
                        0,
                        7,
                    ),
                    index: expr(Expr::Int(0), 9, 9),
                },
                0,
                10,
            ),
            method: "qwe".into(),
            args: vec![expr_raw(Expr::This, 16, 19)],
        },
    );
}

#[test]
fn expr_function_call_with_method_call() {
    assert_expr_parses(
        "function()[0].method()",
        Expr::MethodCall {
            object: expr(
                Expr::ListAccess {
                    list: expr(
                        Expr::FunctionCall { function: "function".into(), args: vec![] },
                        0,
                        9,
                    ),
                    index: expr(Expr::Int(0), 11, 11),
                },
                0,
                12,
            ),
            method: "method".into(),
            args: vec![],
        },
    );

    assert_expr_invalid("function[0]()");
    assert_expr_invalid("(function1, function2)[1]()");
}

#[test]
fn expr_list_access_on_method_call() {
    assert_expr_parses(
        "1.qwe()[0]",
        Expr::ListAccess {
            list: expr(
                Expr::MethodCall {
                    object: expr(Expr::Int(1), 0, 0),
                    method: "qwe".into(),
                    args: vec![],
                },
                0,
                6,
            ),
            index: expr(Expr::Int(0), 8, 8),
        },
    );
}

#[test]
fn expr_new_class_instance() {
    assert_expr_parses(
        "Object()",
        Expr::NewClassInstance { typename: "Object".into(), args: vec![] },
    );
}

#[test]
fn expr_spawn_active() {
    assert_expr_parses(
        "spawn Object()",
        Expr::SpawnActive { typename: "Object".into(), args: vec![] },
    );
}
