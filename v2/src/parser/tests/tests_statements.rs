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
            ifbody: vec![Statement::SExpr(Expr::ExprInt(2))],
            elsebody: vec![],
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
        Statement::SIfElse {
            condition: Expr::ExprInt(1),
            ifbody: vec![Statement::SExpr(Expr::ExprInt(2))],
            elsebody: vec![Statement::SExpr(Expr::ExprInt(3))],
        },
    );
}

#[test]
fn stmt_while() {
    assert_stmt_invalid("while 1; {2}");
    assert_stmt_invalid("while 1 {2}");

    assert_stmt_parses(
        "while 1 {2;}",
        Statement::SWhile {
            condition: Expr::ExprInt(1),
            body: vec![Statement::SExpr(Expr::ExprInt(2))],
        },
    );
}

#[test]
fn stmt_foreach() {
    assert_stmt_parses(
        "foreach obj in (objects) {}",
        Statement::SForeach {
            itemname: String::from("obj"),
            iterable: Expr::ExprIdentifier("objects".into()),
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
        Statement::SWhile {
            condition: Expr::ExprInt(1),
            body: vec![Statement::SContinue, Statement::SBreak],
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
        Statement::SVarDecl(
            Type::TypeIdent(String::from("Actor"), get_test_module_path()),
            String::from("x"),
        ),
    );
}

#[test]
fn stmt_var_decl_equal() {
    assert_stmt_invalid("Int a = 1");
    assert_stmt_invalid("Int 1 = asd;");

    assert_stmt_parses(
        "Actor x = asd;",
        Statement::SVarDeclEqual(
            Type::TypeIdent(String::from("Actor"), get_test_module_path()),
            String::from("x"),
            Expr::ExprIdentifier(String::from("asd")),
        ),
    );
}

#[test]
fn stmt_equal() {
    assert_stmt_invalid("a = 1");
    assert_stmt_invalid("a = asd = q");

    assert_stmt_parses(
        "var = 2;",
        Statement::SAssign {
            left: Expr::ExprIdentifier(String::from("var")),
            right: Expr::ExprInt(2),
        },
    );
}

#[test]
fn stmt_equal_to_list_item() {
    assert_stmt_parses(
        "var[1] = 2;",
        Statement::SAssign {
            left: Expr::ExprListAccess {
                list: Box::new(Expr::ExprIdentifier(String::from("var"))),
                index: Box::new(Expr::ExprInt(1)),
            },
            right: Expr::ExprInt(2),
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
        Statement::SSendMessage {
            active: Expr::ExprFieldAccess {
                object: Box::new(Expr::ExprIdentifier("a".into())),
                field: String::from("x"),
            },
            method: String::from("method"),
            args: vec![],
        },
    );
}
