use crate::tokens::{Token, ScannedToken};
use crate::ast;

struct Parser {

}


pub fn parse(tokens: Vec<ScannedToken>) -> ast::Program {
  ast::Program { imports: vec![], passive: vec![], active: vec![] }
}