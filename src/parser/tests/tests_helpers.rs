use crate::ast::{ExprWithPos, Expr};

use super::super::parser_impl::*;
use super::super::scanner::scan_tokens;

pub type ParsingFunction<T> = fn(&mut Parser) -> ParseResult<T>;


pub fn expr(e: Expr, f: usize, l: usize) -> Box<ExprWithPos> {
    Box::new(expr_raw(e, f, l))
}

pub fn expr_raw(e: Expr, f: usize, l: usize) -> ExprWithPos {
    ExprWithPos { expr: e, pos_first: f, pos_last: l }
}

pub fn parse_and_unwrap<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) -> T {
    let parsed_ast = parse_helper(parsefn, s);

    assert!(
        parsed_ast.is_ok(),
        "Parse error: {:?}",
        parsed_ast.unwrap_err()
    );
    parsed_ast.unwrap()
}

pub fn parse_helper<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) -> ParseResult<T> {
    let tokens = scan_tokens(&String::from(s));
    let mut parser = Parser::create(tokens.unwrap());
    parsefn(&mut parser)
}

pub fn assert_parsing_fails<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) {
    let parsed_ast = parse_helper(parsefn, s);
    assert!(
        parsed_ast.is_err(),
        "{:?} has to fail but parsed to: {:?}",
        s,
        parsed_ast.unwrap()
    );
}
