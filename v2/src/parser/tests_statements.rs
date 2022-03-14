use crate::ast::*;

use super::parser_impl::*;
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
        Statement::SExpr(Expr::ExprBinOp {
            left: Box::new(Expr::ExprNil),
            right: Box::new(Expr::ExprIdentifier(String::from("asd"))),
            op: BinaryOp::IsEqual,
        }),
    );
}

#[test]
fn stmt_return() {
    assert_stmt_invalid("return;");

    assert_stmt_invalid("return 1");
    assert_stmt_invalid("return 2+;");

    assert_stmt_parses("return 1;", Statement::SReturn(Expr::ExprInt(1)));
}

#[test]
fn stmt_if() {
    assert_stmt_invalid("if 1 {2};");
    assert_stmt_invalid("if 1 {2}");

    assert_stmt_parses(
        "if 1 {2;}",
        Statement::SIfElse {
            condition: Expr::ExprInt(1),
            ifbody: vec![Statement::SExpr(Expr::ExprInt(1))],
            elsebody: vec![],
        },
    );
}

#[test]
fn stmt_if_else() {
    assert_stmt_invalid("if 1 {2}; else 3");
    assert_stmt_invalid("if 1 2 else 3");
    assert_stmt_invalid("if 1 {2;} else {3}");
    assert_stmt_invalid("if 1 {2;} else {3;};");

    assert_stmt_parses(
        "if 1 {2;} else {3;}",
        Statement::SIfElse {
            condition: Expr::ExprInt(1),
            ifbody: vec![Statement::SExpr(Expr::ExprInt(1))],
            elsebody: vec![Statement::SExpr(Expr::ExprInt(3))],
        },
    );
}

#[test]
fn stmt_while() {
    assert_stmt_invalid("while 1 {2};");
    assert_stmt_invalid("while 1 {2}");

    assert_stmt_parses(
        "while 1 {2;}",
        Statement::SWhile {
            condition: Expr::ExprInt(1),
            body: vec![Statement::SExpr(Expr::ExprInt(1))],
        },
    );
}

#[test]
fn stmt_for_loop() {
    // TODO: create SList to fit this?
    assert_stmt_parses(
        "for(i=1, ",
        Statement::SWhile {
            condition: Expr::ExprInt(1),
            body: vec![Statement::SExpr(Expr::ExprInt(1))],
        },
    );
}

// TODO: Int x = 1;
// TODO: x = 2;
// TODO: qwe ! qwe();

// statements
// TODO: test array assignment
// TODO
