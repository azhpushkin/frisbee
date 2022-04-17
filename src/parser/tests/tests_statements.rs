use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

fn assert_stmt_parses(s: &str, stmt: Statement) {
    assert_eq!(parse_and_unwrap(Parser::parse_statement, s), stmt);
}

fn assert_stmt_invalid(s: &str) {
    assert_parsing_fails(Parser::parse_statement, s);
}

#[test]
fn stmt_expr() {
    assert_stmt_invalid("(nil) == asd");

    assert_stmt_parses(
        "(nil) == asd;",
        Statement::Expr(Expr::BinOp {
            left: Box::new(Expr::Nil),
            right: Box::new(Expr::Identifier(String::from("asd"))),
            op: BinaryOp::IsEqual,
        }),
    );
}

#[test]
fn stmt_return() {
    assert_stmt_invalid("return 1");
    assert_stmt_invalid("return 2+;");

    assert_stmt_parses("return;", Statement::Return(None));
    assert_stmt_parses("return 1;", Statement::Return(Some(Expr::Int(1))));
}

#[test]
fn stmt_if() {
    assert_stmt_invalid("if 1 {2};");
    assert_stmt_invalid("if 1 {2}");

    assert_stmt_parses(
        "if 1 {2;}",
        Statement::IfElse {
            condition: Expr::Int(1),
            if_body: vec![Statement::Expr(Expr::Int(2))],
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
            condition: Expr::Int(1),
            if_body: vec![Statement::Expr(Expr::Int(2))],
            elif_bodies: vec![],
            else_body: vec![Statement::Expr(Expr::Int(3))],
        },
    );
}

#[test]
fn stmt_if_elif() {
    assert_stmt_parses(
        "if 1 {2;} elif 2 {3;}",
        Statement::IfElse {
            condition: Expr::Int(1),
            if_body: vec![Statement::Expr(Expr::Int(2))],
            elif_bodies: vec![
                (
                    Expr::Int(2),
                    vec![Statement::Expr(Expr::Int(3))],
                ),
            ],
            else_body: vec![],
        },
    );
}

#[test]
fn stmt_if_elif_else() {
    assert_stmt_parses(
        "if 1 {2;} elif 2 {3;} else 4",
        Statement::IfElse {
            condition: Expr::Int(1),
            if_body: vec![Statement::Expr(Expr::Int(2))],
            elif_bodies: vec![
                (
                    Expr::Int(2),
                    vec![Statement::Expr(Expr::Int(3))],
                ),
            ],
            else_body: vec![Statement::Expr(Expr::Int(4))],
        },
    );
}


#[test]
fn stmt_while() {
    assert_stmt_invalid("while 1; {2}");
    assert_stmt_invalid("while 1 {2}");

    assert_stmt_parses(
        "while 1 {2;}",
        Statement::While { condition: Expr::Int(1), body: vec![Statement::Expr(Expr::Int(2))] },
    );
}

#[test]
fn stmt_foreach() {
    assert_stmt_parses(
        "foreach obj in (objects) {}",
        Statement::Foreach {
            itemname: String::from("obj"),
            iterable: Expr::Identifier("objects".into()),
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
            condition: Expr::Int(1),
            body: vec![Statement::Continue, Statement::Break],
        },
    );

    // Only parses when inside of a loop
    assert_stmt_invalid("break;");
    assert_stmt_invalid("continue;");
}

#[test]
fn stmt_var_decl() {
    assert_stmt_invalid("Int a");
    assert_stmt_invalid("Int 1;");

    assert_stmt_parses(
        "Actor x;",
        Statement::VarDecl(Type::Ident(String::from("Actor")), String::from("x")),
    );
}

#[test]
fn stmt_var_decl_equal() {
    assert_stmt_invalid("Int a = 1");
    assert_stmt_invalid("Int 1 = asd;");

    assert_stmt_parses(
        "Actor x = asd;",
        Statement::VarDeclWithAssign(
            Type::Ident(String::from("Actor")),
            String::from("x"),
            Expr::Identifier(String::from("asd")),
        ),
    );
}

#[test]
fn stmt_equal() {
    assert_stmt_invalid("a = 1");
    assert_stmt_invalid("a = asd = q");

    assert_stmt_parses(
        "var = 2;",
        Statement::Assign { left: Expr::Identifier(String::from("var")), right: Expr::Int(2) },
    );
}

#[test]
fn stmt_equal_to_list_item() {
    assert_stmt_parses(
        "var[1] = 2;",
        Statement::Assign {
            left: Expr::ListAccess {
                list: Box::new(Expr::Identifier(String::from("var"))),
                index: Box::new(Expr::Int(1)),
            },
            right: Expr::Int(2),
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
            active: Expr::FieldAccess {
                object: Box::new(Expr::Identifier("a".into())),
                field: String::from("x"),
            },
            method: String::from("method"),
            args: vec![],
        },
    );
}
