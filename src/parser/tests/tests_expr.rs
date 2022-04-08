use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

fn assert_expr_parses(s: &str, t: Expr) {
    assert_eq!(parse_and_unwrap(Parser::parse_expr, s), t);
}

fn assert_expr_invalid(s: &str) {
    assert_parsing_fails(Parser::parse_expr, s);
}

#[test]
fn string_single_and_double_quotes() {
    assert_expr_parses(r#" 'asd' "#, Expr::String(String::from("asd")));
    assert_expr_parses(r#" "asd" "#, Expr::String(String::from("asd")));
}

#[test]
fn operator_simple_equality() {
    assert_expr_parses(
        "(nil) == asd",
        Expr::BinOp {
            left: Box::new(Expr::Nil),
            right: Box::new(Expr::Identifier(String::from("asd"))),
            op: BinaryOp::IsEqual,
        },
    );
}

#[test]
fn operator_expression() {
    assert_expr_parses(
        "1 + asd",
        Expr::BinOp {
            left: Box::new(Expr::Int(1)),
            right: Box::new(Expr::Identifier(String::from("asd"))),
            op: BinaryOp::Plus,
        },
    );

    assert_expr_parses(
        r#"- "hello""#,
        Expr::UnaryOp {
            operand: Box::new(Expr::String(String::from("hello"))),
            op: UnaryOp::Negate,
        },
    );

    assert_expr_parses(
        "true / 23.2",
        Expr::BinOp {
            left: Box::new(Expr::Bool(true)),
            right: Box::new(Expr::Float(23.2)),
            op: BinaryOp::Divide,
        },
    )
}

#[test]
fn expr_minus_minus() {
    assert_expr_parses(
        "-1.0- - 2",
        Expr::BinOp {
            left: Box::new(Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::Float(1.0)),
            }),
            right: Box::new(Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::Int(2)),
            }),
            op: BinaryOp::Minus,
        },
    )
}

#[test]
fn expr_operator_order() {
    assert_expr_parses(
        "1 + 2 * qw2",
        Expr::BinOp {
            left: Box::new(Expr::Int(1)),
            right: Box::new(Expr::BinOp {
                left: Box::new(Expr::Int(2)),
                right: Box::new(Expr::Identifier(String::from("qw2"))),
                op: BinaryOp::Multiply,
            }),
            op: BinaryOp::Plus,
        },
    )
}

#[test]
fn expr_simple_groups() {
    assert_expr_parses("(1)", Expr::Int(1));
    assert_expr_parses(
        "(-(-3))",
        Expr::UnaryOp {
            op: UnaryOp::Negate,
            operand: Box::new(Expr::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::Int(3)),
            }),
        },
    );
}

#[test]
fn expr_operator_order_with_grouping() {
    assert_expr_parses(
        "2 * (1 + qw2)",
        Expr::BinOp {
            left: Box::new(Expr::Int(2)),
            right: Box::new(Expr::BinOp {
                left: Box::new(Expr::Int(1)),
                right: Box::new(Expr::Identifier(String::from("qw2"))),
                op: BinaryOp::Plus,
            }),
            op: BinaryOp::Multiply,
        },
    );

    assert_expr_parses(
        "(1 + qw2) * 2",
        Expr::BinOp {
            left: Box::new(Expr::BinOp {
                left: Box::new(Expr::Int(1)),
                right: Box::new(Expr::Identifier(String::from("qw2"))),
                op: BinaryOp::Plus,
            }),
            right: Box::new(Expr::Int(2)),
            op: BinaryOp::Multiply,
        },
    );
}

#[test]
fn expr_tuple() {
    assert_expr_parses(
        "(1, 2.0, ad)",
        Expr::TupleValue(vec![
            Expr::Int(1),
            Expr::Float(2.0),
            Expr::Identifier(String::from("ad")),
        ]),
    );

    assert_expr_parses(
        "((1, 2), (3, 4))",
        Expr::TupleValue(vec![
            Expr::TupleValue(vec![Expr::Int(1), Expr::Int(2)]),
            Expr::TupleValue(vec![Expr::Int(3), Expr::Int(4)]),
        ]),
    );

    // single-element tuple is simplified to just an element
    assert_expr_parses("(1, )", Expr::Int(1));
}

#[test]
fn expr_group_and_tuple_mixed() {
    assert_expr_parses(
        "((1, 2) + (3, 4))",
        Expr::BinOp {
            left: Box::new(Expr::TupleValue(vec![
                Expr::Int(1),
                Expr::Int(2),
            ])),
            right: Box::new(Expr::TupleValue(vec![
                Expr::Int(3),
                Expr::Int(4),
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
}

#[test]
fn expr_list_value() {
    assert_expr_parses("[]", Expr::ListValue(vec![]));

    assert_expr_parses(
        "[1, new]",
        Expr::ListValue(vec![
            Expr::Int(1),
            Expr::Identifier(String::from("new")),
        ]),
    );

    // trailing comma is allowed
    assert_expr_parses(
        "[nil, 2.0,]",
        Expr::ListValue(vec![Expr::Nil, Expr::Float(2.0)]),
    );

    assert_expr_invalid("[, ]");
}

#[test]
fn expr_list_access() {
    assert_expr_parses(
        "asd[2]",
        Expr::ListAccess {
            list: Box::new(Expr::Identifier(String::from("asd"))),
            index: Box::new(Expr::Int(2)),
        },
    );
}

#[test]
fn expr_list_access_chained() {
    assert_expr_parses(
        "asd[2][0]",
        Expr::ListAccess {
            list: Box::new(Expr::ListAccess {
                list: Box::new(Expr::Identifier(String::from("asd"))),
                index: Box::new(Expr::Int(2)),
            }),
            index: Box::new(Expr::Int(0)),
        },
    );
}

#[test]
fn expr_field_access() {
    assert_expr_parses(
        "(1).qwe",
        Expr::FieldAccess { object: Box::new(Expr::Int(1)), field: String::from("qwe") },
    );

    assert_expr_invalid("obj.(field)");
}

#[test]
fn expr_own_field_access() {
    assert_expr_parses(
        "@qwe",
        Expr::OwnFieldAccess { field: String::from("qwe") },
    );
}

#[test]
fn expr_func() {
    assert_expr_parses(
        "(function) (1, )",
        Expr::FunctionCall { function: String::from("function"), args: vec![Expr::Int(1)] },
    );

    assert_expr_invalid("function.()");
    assert_expr_invalid("function()()");
}

#[test]
fn expr_field_access_chained_with_method_call() {
    assert_expr_parses(
        "obj.field.method()",
        Expr::MethodCall {
            object: Box::new(Expr::FieldAccess {
                object: Box::new(Expr::Identifier(String::from("obj"))),
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
        Expr::MethodCall {
            object: Box::new(Expr::Int(1)),
            method: String::from("qwe"),
            args: vec![],
        },
    );
}

#[test]
fn expr_own_method_call() {
    assert_expr_parses(
        "(@qwe())",
        Expr::OwnMethodCall { method: String::from("qwe"), args: vec![] },
    );
}

#[test]
fn expr_method_call_with_args() {
    assert_expr_parses(
        "asd.qwe(1, )",
        Expr::MethodCall {
            object: Box::new(Expr::Identifier(String::from("asd"))),
            method: String::from("qwe"),
            args: vec![Expr::Int(1)],
        },
    );

    assert_expr_parses(
        "asd.qwe(1, true, this)",
        Expr::MethodCall {
            object: Box::new(Expr::Identifier(String::from("asd"))),
            method: String::from("qwe"),
            args: vec![Expr::Int(1), Expr::Bool(true), Expr::This],
        },
    );
}

#[test]
fn expr_own_method_call_and_field_access() {
    assert_expr_parses(
        "@qwe().next(@field)",
        Expr::MethodCall {
            object: Box::new(Expr::OwnMethodCall { method: String::from("qwe"), args: vec![] }),
            method: String::from("next"),
            args: vec![Expr::OwnFieldAccess { field: "field".into() }],
        },
    );
}

#[test]
fn expr_method_call_chained() {
    assert_expr_parses(
        "(1, 2).qwe().asd()",
        Expr::MethodCall {
            object: Box::new(Expr::MethodCall {
                object: Box::new(Expr::TupleValue(vec![
                    Expr::Int(1),
                    Expr::Int(2),
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
    let expected = Expr::MethodCall {
        object: Box::new(Expr::ListValue(vec![
            Expr::Identifier(String::from("asd")),
            Expr::Int(2),
        ])),
        method: String::from("qwe"),
        args: vec![Expr::This],
    };

    assert_expr_parses("([asd, 2]).qwe(this)", expected);
}

#[test]
fn expr_call_method_on_list_access() {
    assert_expr_parses(
        "[asd, 2][0].qwe(this)",
        Expr::MethodCall {
            object: Box::new(Expr::ListAccess {
                list: Box::new(Expr::ListValue(vec![
                    Expr::Identifier(String::from("asd")),
                    Expr::Int(2),
                ])),
                index: Box::new(Expr::Int(0)),
            }),
            method: String::from("qwe"),
            args: vec![Expr::This],
        },
    );
}

#[test]
fn expr_function_call_with_method_call() {
    assert_expr_parses(
        "function()[0].method()",
        Expr::MethodCall {
            object: Box::new(Expr::ListAccess {
                list: Box::new(Expr::FunctionCall {
                    function: String::from("function"),
                    args: vec![],
                }),
                index: Box::new(Expr::Int(0)),
            }),
            method: String::from("method"),
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
            list: Box::new(Expr::MethodCall {
                object: Box::new(Expr::Int(1)),
                method: String::from("qwe"),
                args: vec![],
            }),
            index: Box::new(Expr::Int(0)),
        },
    );
}

#[test]
fn expr_new_class_instance() {
    assert_expr_parses(
        "Object()",
        Expr::NewClassInstance { typename: String::from("Object"), args: vec![] },
    );
}

#[test]
fn expr_spawn_active() {
    assert_expr_parses(
        "spawn Object()",
        Expr::SpawnActive { typename: String::from("Object"), args: vec![] },
    );
}
