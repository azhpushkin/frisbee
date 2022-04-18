use crate::ast::FileAst;

mod helpers;
pub mod scanner;
pub mod parser_impl;

// mod tests;

// pass this types through mod.rs
pub type ParseError = parser_impl::ParseError;
pub type ParseResult<T> = parser_impl::ParseResult<T>;

pub fn parse(tokens: Vec<scanner::ScannedToken>) -> ParseResult<FileAst> {
    let mut parser = parser_impl::Parser::create(tokens);
    parser.parse_top_level()
}
