mod parser_impl;

#[rustfmt::skip]
#[cfg(test)] mod tests; // mark tests file to improve cargo awareness

use crate::ast::Program;
use crate::tokens::ScannedToken;

// pass this types through mod.rs
pub type ParseError = parser_impl::ParseError;
pub type ParseResult<T> = parser_impl::ParseResult<T>;

pub fn parse(tokens: Vec<ScannedToken>) -> ParseResult<Program> {
    let mut parser = parser_impl::Parser::create(tokens);
    parser.parse_top_level()
}
