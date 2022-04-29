use crate::ast::FileAst;

mod helpers;
pub mod parser;
pub mod scanner;

mod tests;

// pass this types through mod.rs
pub type ParseError = parser::ParseError;
pub type ParseResult<T> = parser::ParseResult<T>;

pub fn parse(tokens: Vec<scanner::ScannedToken>) -> ParseResult<FileAst> {
    let mut parser = parser::Parser::create(tokens);
    parser.parse_top_level()
}
