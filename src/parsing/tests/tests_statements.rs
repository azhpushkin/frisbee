use crate::ast::parsed::*;
use crate::types::Type;

use super::super::parser::*;
use super::tests_helpers::*;

fn assert_stmt_parses(s: &str, stmt: Statement) {
    let res = parse_and_unwrap(Parser::parse_statement, s);
    assert_eq!(res.statement, stmt);
}

fn assert_stmt_invalid(s: &str) {
    assert_parsing_fails(Parser::parse_statement, s);
}

#[test]
fn stmt_expr() {
    assert_stmt_invalid("(nil) == asd");

    assert_stmt_parses(
        "(nil) == asd;",
        Statement::Expr(expr_raw(
            Expr::BinOp {
                left: expr(Expr::Nil, 1, 3),
                right: expr(Expr::Identifier("asd".into()), 9, 11),
                op: BinaryOp::IsEqual,
            },
            0,
            11,
        )),
    );
}

#[test]
fn stmt_return() {
    assert_stmt_invalid("return 1");
    assert_stmt_invalid("return 2+;");

    assert_stmt_parses("return;", Statement::Return(None));
    assert_stmt_parses(
        "return 1;",
        Statement::Return(Some(expr_raw(Expr::Int(1), 7, 7))),
    );
}

#[test]
fn stmt_if() {
    assert_stmt_invalid("if 1 {2};");
    assert_stmt_invalid("if 1 {2}");

    assert_stmt_parses(
        "if 1 {2;}",
        Statement::IfElse {
            condition: expr_raw(Expr::Int(1), 3, 3),
            if_body: vec![stmt(Statement::Expr(expr_raw(Expr::Int(2), 6, 6)), 6)],
            elif_bodies: vec![],
            else_body: vec![],
        },
    );
}

#[test]
fn stmt_if_else() {
    assert_stmt_invalid("if 1 {2}; else 3");
    assert_stmt_invalid("if 1 2 else 3");
    assert_stmt_invalid("if 1 {2;} else {3}");

    assert_stmt_parses(
        "if 1 {2;} else {3;}",
        Statement::IfElse {
            condition: expr_raw(Expr::Int(1), 3, 3),
            if_body: vec![stmt(Statement::Expr(expr_raw(Expr::Int(2), 6, 6)), 6)],
            elif_bodies: vec![],
            else_body: vec![stmt(Statement::Expr(expr_raw(Expr::Int(3), 16, 16)), 16)],
        },
    );
}

#[test]
fn stmt_if_elif() {
    assert_stmt_parses(
        "if 1 {2;} elif 2 {3;}",
        Statement::IfElse {
            condition: expr_raw(Expr::Int(1), 3, 3),
            if_body: vec![stmt(Statement::Expr(expr_raw(Expr::Int(2), 6, 6)), 6)],
            elif_bodies: vec![(
                expr_raw(Expr::Int(2), 15, 15),
                vec![stmt(Statement::Expr(expr_raw(Expr::Int(3), 18, 18)), 18)],
            )],
            else_body: vec![],
        },
    );
}

#[test]
fn stmt_if_elif_else() {
    assert_stmt_parses(
        "if 1 {2;} elif 2 {3;} else {4; }",
        Statement::IfElse {
            condition: expr_raw(Expr::Int(1), 3, 3),
            if_body: vec![stmt(Statement::Expr(expr_raw(Expr::Int(2), 6, 6)), 6)],
            elif_bodies: vec![(
                expr_raw(Expr::Int(2), 15, 15),
                vec![stmt(Statement::Expr(expr_raw(Expr::Int(3), 18, 18)), 18)],
            )],
            else_body: vec![stmt(Statement::Expr(expr_raw(Expr::Int(4), 28, 28)), 28)],
        },
    );
}

#[test]
fn stmt_while() {
    assert_stmt_invalid("while 1; {2}");
    assert_stmt_invalid("while 1 {2}");

    assert_stmt_parses(
        "while 1 {2;}",
        Statement::While {
            condition: expr_raw(Expr::Int(1), 6, 6),
            body: vec![stmt(Statement::Expr(expr_raw(Expr::Int(2), 9, 9)), 9)],
        },
    );
}

#[test]
fn stmt_foreach() {
    assert_stmt_parses(
        "foreach obj in (objects) {}",
        Statement::Foreach {
            item_name: "obj".into(),
            iterable: expr_raw(Expr::Identifier("objects".into()), 16, 22),
            body: vec![],
        },
    );

    assert_stmt_invalid("foreach (obj in objects) {}");
    assert_stmt_invalid("foreach (obj) in objects {}");
    assert_stmt_invalid("foreach Obj in objects {}");
    assert_stmt_invalid("foreach Obj in objects {}");
}

#[test]
fn stmt_break_and_continue() {
    assert_stmt_parses(
        "while 1 {continue; break;}",
        Statement::While {
            condition: expr_raw(Expr::Int(1), 6, 6),
            body: vec![stmt(Statement::Continue, 9), stmt(Statement::Break, 19)],
        },
    );
}

#[test]
fn stmt_var_decl() {
    assert_stmt_invalid("Int a");
    assert_stmt_invalid("Int 1;");

    assert_stmt_parses(
        "Actor x;",
        Statement::VarDecl(Type::Custom("Actor".into()), "x".into()),
    );
}

#[test]
fn stmt_var_decl_equal() {
    assert_stmt_invalid("Int a = 1");
    assert_stmt_invalid("Int 1 = asd;");

    assert_stmt_parses(
        "Actor x = asd;",
        Statement::VarDeclWithAssign(
            Type::Custom("Actor".into()),
            "x".into(),
            expr_raw(Expr::Identifier("asd".into()), 10, 12),
        ),
    );
}

#[test]
fn stmt_equal() {
    assert_stmt_invalid("a = 1");
    assert_stmt_invalid("a = asd = q");

    assert_stmt_parses(
        "var = 2;",
        Statement::Assign {
            left: expr_raw(Expr::Identifier("var".into()), 0, 2),
            right: expr_raw(Expr::Int(2), 6, 6),
        },
    );
}

#[test]
fn stmt_equal_to_list_item() {
    assert_stmt_parses(
        "var[1] = 2;",
        Statement::Assign {
            left: expr_raw(
                Expr::ListAccess {
                    list: expr(Expr::Identifier("var".into()), 0, 2),
                    index: expr(Expr::Int(1), 4, 4),
                },
                0,
                5,
            ),
            right: expr_raw(Expr::Int(2), 9, 9),
        },
    );
}

#[test]
fn stmt_send_message() {
    assert_stmt_invalid("a ! 1;");
    assert_stmt_invalid("a ! asd;");
    assert_stmt_invalid("a ! ads()");

    assert_stmt_parses(
        "a.x ! method();",
        Statement::SendMessage {
            active: expr_raw(
                Expr::FieldAccess {
                    object: expr(Expr::Identifier("a".into()), 0, 0),
                    field: "x".into(),
                },
                0,
                2,
            ),
            method: "method".into(),
            args: vec![],
        },
    );
}
