use crate::ast::FileAst;
use crate::scanner::ScannedToken;

mod helpers;
pub mod parser_impl;

mod tests;

// pass this types through mod.rs
pub type ParseError = parser_impl::ParseError;
pub type ParseResult<T> = parser_impl::ParseResult<T>;

pub fn parse(tokens: Vec<ScannedToken>) -> ParseResult<FileAst> {
    let mut parser = parser_impl::Parser::create(tokens);
    parser.parse_top_level()
}
