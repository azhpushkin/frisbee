use crate::tokens::*;
use crate::ast;

struct Parser {
  tokens: Vec<ScannedToken>,
  position: usize
}

impl Parser {
  fn create(tokens: Vec<ScannedToken>) -> Parser {
    Parser { tokens, position: 0 }
  }
  
  fn token_ahead(&self, ahead: usize) -> &Token {
    match self.tokens.get(self.position + ahead) {
      Some((token, pos)) => token,
      None => &Token::EOF,
    }
  }

  fn is_finished(&self) -> bool {
    self.position >= self.tokens.len()
  }
}


pub fn parse(tokens: Vec<ScannedToken>) -> ast::Program {
  let mut parser = Parser::create(tokens);
  










  ast::Program { imports: vec![], passive: vec![], active: vec![] }
}