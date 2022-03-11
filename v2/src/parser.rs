use crate::tokens::*;
use crate::ast::*;


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

  fn parse(&mut self) -> Program {
    let mut program = Program { imports: vec![], passive: vec![], active: vec![] };

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
        Token::EOF => { break; },
        t => {
          panic!("Only imports and object declarations are allowed, but received {}", t)
        }
      }
    
    }
    program
  }
  
  fn parse_import(&mut self) -> ImportDecl {
    match self.consume_token() {
      (Token::Identifier(module_name), _) => {
        self.consume_and_check(Token::Import);
        
        let (typename, _) = self.consume_token();
        let res: String;
        match typename {
          Token::Identifier(s) => {res = s}
          _ => panic!("asd")
        };
        ImportDecl { module: module_name, typenames: vec![res, ] }

      }
      c => panic!("I'm so sorry.. {:?}", c)
    }

    

  }

  fn parse_object(&mut self) -> ObjectDecl {
    ObjectDecl { is_active: true, name: String::from("Obj"), fields: vec![], methods: vec![] }
  }
}


pub fn parse(tokens: Vec<ScannedToken>) -> Program {
  let mut parser = Parser::create(tokens);
  parser.parse()

  
}



mod tests {
  use super::*;
  use crate::tokens::scan_tokens;

  fn get_ast_helper(s: &str) -> Program {
    let tokens = scan_tokens(String::from(s));
    parse(tokens)
  }

  #[test]
  fn simple_import() {
    assert_eq!(
      get_ast_helper("from module import Actor"),
      Program {
        imports: vec![ImportDecl { module: String::from("module"), typenames: vec![String::from("Actor")] }],
        passive: vec![], active: vec![]
      }
    );

    // assert_eq!(
    //   get_ast_helper("from some2 import Hello, There"),
    //   Program {
    //     imports: vec![ImportDecl { 
    //       module: String::from("module"),
    //       typenames: vec![String::from("Hello"), String::from("There")]
    //     }],
    //     passive: vec![], active: vec![]
    //   }
    // );
  }
}