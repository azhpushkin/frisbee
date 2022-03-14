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

// statements
// TODO: test array assignment
// TODO
