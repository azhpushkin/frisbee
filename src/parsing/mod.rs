use crate::ast::parsed::FileAst;

mod helpers;
pub(self) mod parser;
pub mod scanner;

mod tests;

// pass this types through mod.rs
pub type ParseError = parser::ParseError;

pub fn parse_file(tokens: Vec<scanner::ScannedToken>) -> Result<FileAst, ParseError> {
    let mut parser = parser::Parser::create(tokens);
    parser.parse_top_level()
}
