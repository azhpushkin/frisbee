mod helpers;
mod parser_impl;

#[rustfmt::skip] #[cfg(test)] mod tests_helpers;
#[rustfmt::skip] #[cfg(test)] mod tests_top_level;
#[rustfmt::skip] #[cfg(test)] mod tests_types;
#[rustfmt::skip] #[cfg(test)] mod tests_expr;

use crate::ast::Program;
use crate::tokens::ScannedToken;

// pass this types through mod.rs
pub type ParseError = parser_impl::ParseError;
pub type ParseResult<T> = parser_impl::ParseResult<T>;

pub fn parse(tokens: Vec<ScannedToken>) -> ParseResult<Program> {
    let mut parser = parser_impl::Parser::create(tokens);
    parser.parse_top_level()
}
