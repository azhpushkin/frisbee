mod parser_impl;
mod tests;

use crate::tokens::ScannedToken;
use crate::ast::Program;


// pass this types through mod.rs
pub type ParseError = parser_impl::ParseError;
pub type ParseResult<T> = parser_impl::ParseResult<T>;


pub fn parse(tokens: Vec<ScannedToken>) -> ParseResult<Program> {
    let mut parser = parser_impl::Parser::create(tokens);
    parser.parse_top_level()
}