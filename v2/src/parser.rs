use std::iter::Scan;

use crate::tokens::*;
use crate::ast;


fn token_type(st: ScannedToken) -> Token {
  match st {
    (token, _pos) => token
  }
}


struct Parser {
  tokens: Vec<ScannedToken>,
  position: usize
}

impl Parser {
  fn create(tokens: Vec<ScannedToken>) -> Parser {
    Parser { tokens, position: 0 }
  }
  
  fn rel_token(&self, rel_pos: isize) -> &ScannedToken {
    let pos = if rel_pos < 0 {
      self.position - (rel_pos.abs() as usize)
    } else {self.position + (rel_pos as usize)};

    match self.tokens.get(pos) {
      Some(x) => x,
      None => &(Token::EOF, 0), // 0 here is strange
    }
  }

  fn consume_token(&mut self) -> ScannedToken {
    self.position += 1;
    self.rel_token(-1).clone()
  }

  fn consume_and_check(&mut self, token: Token) {
    match self.consume_token() {
      (t, p) if t.eq(&token) => (),
      _ => panic!("Wrong char here!")
    }
  }

  fn is_finished(&self) -> bool {
    self.position >= self.tokens.len()
  }

  fn parse(&mut self) -> ast::Program {
    let mut program = ast::Program { imports: vec![], passive: vec![], active: vec![] };

    while !self.is_finished() {
      let token = self.consume_token().clone();

      match token_type(token) {
        Token::From => {
          program.imports.push(self.parse_import())
        },
        Token::Active => {
          program.active.push(self.parse_object())
        }
        Token::Passive => {
          program.passive.push(self.parse_object())
        }
        t => {
          panic!("Only imports and object declarations are allowed, but received {}", t)
        }
      }
    
    }
    program
  }
  
  fn parse_import(&mut self) -> ast::ImportDecl {
    match self.consume_token() {
      (Token::String(module_name), _) => {
        self.consume_and_check(Token::Import);
        
        let (typename, _) = self.consume_token();
        let res: String;
        match typename {
          Token::String(s) => {res = s}
          _ => panic!("asd")
        };
        ast::ImportDecl { module: module_name, typenames: vec![typename, ] }

      }
      _ => panic!("I'm so sorry..")
    }

    

  }

  fn parse_object(&mut self) -> ast::ObjectDecl {
    ast::ObjectDecl { is_active: true, name: "Obj", fields: vec![], methods: vec![] }
  }
}


pub fn parse(tokens: Vec<ScannedToken>) -> ast::Program {
  let mut parser = Parser::create(tokens);
  parser.parse()

  
}



mod tests {
  
}