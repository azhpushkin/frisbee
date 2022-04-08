use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

fn assert_expr_parses(s: &str, t: ExprRaw) {
    assert_eq!(parse_and_unwrap(Parser::parse_expr, s), t);
}

fn assert_expr_invalid(s: &str) {
    assert_parsing_fails(Parser::parse_expr, s);
}

#[test]
fn string_single_and_double_quotes() {
    assert_expr_parses(r#" 'asd' "#, ExprRaw::String(String::from("asd")));
    assert_expr_parses(r#" "asd" "#, ExprRaw::String(String::from("asd")));
}

#[test]
fn operator_simple_equality() {
    assert_expr_parses(
        "(nil) == asd",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::Nil),
            right: Box::new(ExprRaw::Identifier(String::from("asd"))),
            op: BinaryOp::IsEqual,
        },
    );
}

#[test]
fn operator_expression() {
    assert_expr_parses(
        "1 + asd",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::Int(1)),
            right: Box::new(ExprRaw::Identifier(String::from("asd"))),
            op: BinaryOp::Plus,
        },
    );

    assert_expr_parses(
        r#"- "hello""#,
        ExprRaw::UnaryOp {
            operand: Box::new(ExprRaw::String(String::from("hello"))),
            op: UnaryOp::Negate,
        },
    );

    assert_expr_parses(
        "true / 23.2",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::Bool(true)),
            right: Box::new(ExprRaw::Float(23.2)),
            op: BinaryOp::Divide,
        },
    )
}

#[test]
fn expr_minus_minus() {
    assert_expr_parses(
        "-1.0- - 2",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(ExprRaw::Float(1.0)),
            }),
            right: Box::new(ExprRaw::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(ExprRaw::Int(2)),
            }),
            op: BinaryOp::Minus,
        },
    )
}

#[test]
fn expr_operator_order() {
    assert_expr_parses(
        "1 + 2 * qw2",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::Int(1)),
            right: Box::new(ExprRaw::BinOp {
                left: Box::new(ExprRaw::Int(2)),
                right: Box::new(ExprRaw::Identifier(String::from("qw2"))),
                op: BinaryOp::Multiply,
            }),
            op: BinaryOp::Plus,
        },
    )
}

#[test]
fn expr_simple_groups() {
    assert_expr_parses("(1)", ExprRaw::Int(1));
    assert_expr_parses(
        "(-(-3))",
        ExprRaw::UnaryOp {
            op: UnaryOp::Negate,
            operand: Box::new(ExprRaw::UnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(ExprRaw::Int(3)),
            }),
        },
    );
}

#[test]
fn expr_operator_order_with_grouping() {
    assert_expr_parses(
        "2 * (1 + qw2)",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::Int(2)),
            right: Box::new(ExprRaw::BinOp {
                left: Box::new(ExprRaw::Int(1)),
                right: Box::new(ExprRaw::Identifier(String::from("qw2"))),
                op: BinaryOp::Plus,
            }),
            op: BinaryOp::Multiply,
        },
    );

    assert_expr_parses(
        "(1 + qw2) * 2",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::BinOp {
                left: Box::new(ExprRaw::Int(1)),
                right: Box::new(ExprRaw::Identifier(String::from("qw2"))),
                op: BinaryOp::Plus,
            }),
            right: Box::new(ExprRaw::Int(2)),
            op: BinaryOp::Multiply,
        },
    );
}

#[test]
fn expr_tuple() {
    assert_expr_parses(
        "(1, 2.0, ad)",
        ExprRaw::TupleValue(vec![
            ExprRaw::Int(1),
            ExprRaw::Float(2.0),
            ExprRaw::Identifier(String::from("ad")),
        ]),
    );

    assert_expr_parses(
        "((1, 2), (3, 4))",
        ExprRaw::TupleValue(vec![
            ExprRaw::TupleValue(vec![ExprRaw::Int(1), ExprRaw::Int(2)]),
            ExprRaw::TupleValue(vec![ExprRaw::Int(3), ExprRaw::Int(4)]),
        ]),
    );

    // single-element tuple is simplified to just an element
    assert_expr_parses("(1, )", ExprRaw::Int(1));
}

#[test]
fn expr_group_and_tuple_mixed() {
    assert_expr_parses(
        "((1, 2) + (3, 4))",
        ExprRaw::BinOp {
            left: Box::new(ExprRaw::TupleValue(vec![
                ExprRaw::Int(1),
                ExprRaw::Int(2),
            ])),
            right: Box::new(ExprRaw::TupleValue(vec![
                ExprRaw::Int(3),
                ExprRaw::Int(4),
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
    assert_expr_parses("[]", ExprRaw::ListValue(vec![]));

    assert_expr_parses(
        "[1, new]",
        ExprRaw::ListValue(vec![
            ExprRaw::Int(1),
            ExprRaw::Identifier(String::from("new")),
        ]),
    );

    // trailing comma is allowed
    assert_expr_parses(
        "[nil, 2.0,]",
        ExprRaw::ListValue(vec![ExprRaw::Nil, ExprRaw::Float(2.0)]),
    );

    assert_expr_invalid("[, ]");
}

#[test]
fn expr_list_access() {
    assert_expr_parses(
        "asd[2]",
        ExprRaw::ListAccess {
            list: Box::new(ExprRaw::Identifier(String::from("asd"))),
            index: Box::new(ExprRaw::Int(2)),
        },
    );
}

#[test]
fn expr_list_access_chained() {
    assert_expr_parses(
        "asd[2][0]",
        ExprRaw::ListAccess {
            list: Box::new(ExprRaw::ListAccess {
                list: Box::new(ExprRaw::Identifier(String::from("asd"))),
                index: Box::new(ExprRaw::Int(2)),
            }),
            index: Box::new(ExprRaw::Int(0)),
        },
    );
}

#[test]
fn expr_field_access() {
    assert_expr_parses(
        "(1).qwe",
        ExprRaw::FieldAccess { object: Box::new(ExprRaw::Int(1)), field: String::from("qwe") },
    );

    assert_expr_invalid("obj.(field)");
}

#[test]
fn expr_own_field_access() {
    assert_expr_parses(
        "@qwe",
        ExprRaw::OwnFieldAccess { field: String::from("qwe") },
    );
}

#[test]
fn expr_func() {
    assert_expr_parses(
        "(function) (1, )",
        ExprRaw::FunctionCall { function: String::from("function"), args: vec![ExprRaw::Int(1)] },
    );

    assert_expr_invalid("function.()");
    assert_expr_invalid("function()()");
}

#[test]
fn expr_field_access_chained_with_method_call() {
    assert_expr_parses(
        "obj.field.method()",
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::FieldAccess {
                object: Box::new(ExprRaw::Identifier(String::from("obj"))),
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
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::Int(1)),
            method: String::from("qwe"),
            args: vec![],
        },
    );
}

#[test]
fn expr_own_method_call() {
    assert_expr_parses(
        "(@qwe())",
        ExprRaw::OwnMethodCall { method: String::from("qwe"), args: vec![] },
    );
}

#[test]
fn expr_method_call_with_args() {
    assert_expr_parses(
        "asd.qwe(1, )",
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::Identifier(String::from("asd"))),
            method: String::from("qwe"),
            args: vec![ExprRaw::Int(1)],
        },
    );

    assert_expr_parses(
        "asd.qwe(1, true, this)",
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::Identifier(String::from("asd"))),
            method: String::from("qwe"),
            args: vec![ExprRaw::Int(1), ExprRaw::Bool(true), ExprRaw::This],
        },
    );
}

#[test]
fn expr_own_method_call_and_field_access() {
    assert_expr_parses(
        "@qwe().next(@field)",
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::OwnMethodCall { method: String::from("qwe"), args: vec![] }),
            method: String::from("next"),
            args: vec![ExprRaw::OwnFieldAccess { field: "field".into() }],
        },
    );
}

#[test]
fn expr_method_call_chained() {
    assert_expr_parses(
        "(1, 2).qwe().asd()",
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::MethodCall {
                object: Box::new(ExprRaw::TupleValue(vec![
                    ExprRaw::Int(1),
                    ExprRaw::Int(2),
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
    let expected = ExprRaw::MethodCall {
        object: Box::new(ExprRaw::ListValue(vec![
            ExprRaw::Identifier(String::from("asd")),
            ExprRaw::Int(2),
        ])),
        method: String::from("qwe"),
        args: vec![ExprRaw::This],
    };

    assert_expr_parses("([asd, 2]).qwe(this)", expected);
}

#[test]
fn expr_call_method_on_list_access() {
    assert_expr_parses(
        "[asd, 2][0].qwe(this)",
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::ListAccess {
                list: Box::new(ExprRaw::ListValue(vec![
                    ExprRaw::Identifier(String::from("asd")),
                    ExprRaw::Int(2),
                ])),
                index: Box::new(ExprRaw::Int(0)),
            }),
            method: String::from("qwe"),
            args: vec![ExprRaw::This],
        },
    );
}

#[test]
fn expr_function_call_with_method_call() {
    assert_expr_parses(
        "function()[0].method()",
        ExprRaw::MethodCall {
            object: Box::new(ExprRaw::ListAccess {
                list: Box::new(ExprRaw::FunctionCall {
                    function: String::from("function"),
                    args: vec![],
                }),
                index: Box::new(ExprRaw::Int(0)),
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
        ExprRaw::ListAccess {
            list: Box::new(ExprRaw::MethodCall {
                object: Box::new(ExprRaw::Int(1)),
                method: String::from("qwe"),
                args: vec![],
            }),
            index: Box::new(ExprRaw::Int(0)),
        },
    );
}

#[test]
fn expr_new_class_instance() {
    assert_expr_parses(
        "Object()",
        ExprRaw::NewClassInstance { typename: String::from("Object"), args: vec![] },
    );
}

#[test]
fn expr_spawn_active() {
    assert_expr_parses(
        "spawn Object()",
        ExprRaw::SpawnActive { typename: String::from("Object"), args: vec![] },
    );
}
