mod helpers;
mod parser_impl;

mod tests;

use crate::ast::{FileAst, ModulePathAlias};
use crate::scanner::ScannedToken;

// pass this types through mod.rs
pub type ParseError = parser_impl::ParseError;
pub type ParseResult<T> = parser_impl::ParseResult<T>;

pub fn parse(tokens: Vec<ScannedToken>, scope: ModulePathAlias) -> ParseResult<FileAst> {
    let mut parser = parser_impl::Parser::create(tokens, &scope);
    parser.parse_top_level()
}
