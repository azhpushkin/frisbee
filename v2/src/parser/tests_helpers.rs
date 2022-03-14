use super::parser_impl::*;
use crate::tokens::scan_tokens;

pub type ParsingFunction<T> = fn(&mut Parser) -> ParseResult<T>;

pub fn parse_helper<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) -> T {
    let tokens = scan_tokens(&String::from(s));
    let mut parser = Parser::create(tokens);
    let parsed_ast = parsefn(&mut parser);

    assert!(
        parsed_ast.is_ok(),
        "Parse error: {:?}",
        parsed_ast.unwrap_err()
    );
    parsed_ast.unwrap()
}

pub fn assert_type_parsing_fails<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) {
    let tokens = scan_tokens(&String::from(s));
    let mut parser = Parser::create(tokens);
    let parsed_ast = parsefn(&mut parser);

    assert!(parsed_ast.is_err(), "Parsed to: {:?}", parsed_ast.unwrap());
}
