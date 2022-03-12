use crate::ast::*;
use crate::tokens::*;

struct Parser {
    tokens: Vec<ScannedToken>,
    position: usize,
}

type ParseError = (ScannedToken, &'static str);

macro_rules! extract_result_if_ok {
    ($parse_result:expr) => {
        match $parse_result {
            Ok(res) => res,
            Err(t) => {
                // Re-wrap pf parsing error is required to coerce type
                // from Result<T, ParseError> to Result<Program, ParseError>
                return Err(t);
            }
        }
    };
}

macro_rules! consume_and_check {
    ($self:ident, $token:expr) => {
        match $self.consume_token() {
            (t, _) if t.eq(&$token) => (),
            t => {
                return Err((t, "Unexpected token"));
            }
        }
    };
}

macro_rules! consume_and_check_ident {
    ($self:ident) => {
        match $self.consume_token() {
            (Token::Identifier(s), _) => s,
            t => {
                return Err((t, "Unexpected token (expected identifier)"));
            }
        }
    };
}

macro_rules! consume_and_check_type_ident {
    ($self:ident) => {
        match $self.consume_token() {
            (Token::TypeIdentifier(s), _) => s,
            t => {
                return Err((t, "Unexpected token (expected identifier)"));
            }
        }
    };
}

impl Parser {
    fn create(tokens: Vec<ScannedToken>) -> Parser {
        Parser {
            tokens,
            position: 0,
        }
    }

    fn rel_token(&self, rel_pos: isize) -> &ScannedToken {
        let pos = if rel_pos < 0 {
            self.position - (rel_pos.abs() as usize)
        } else {
            self.position + (rel_pos as usize)
        };

        match self.tokens.get(pos) {
            Some(x) => x,
            None => &(Token::EOF, 0), // 0 here is strange
        }
    }

    fn consume_token(&mut self) -> ScannedToken {
        self.position += 1;
        self.rel_token(-1).clone()
    }

    fn rel_token_check(&mut self, rel_pos: isize, token: Token) -> bool {
        match self.rel_token(rel_pos) {
            (x, _) => token.eq(x),
        }
    }

    fn is_finished(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn parse_top_level(&mut self) -> Result<Program, ParseError> {
        let mut program = Program {
            imports: vec![],
            passive: vec![],
            active: vec![],
        };

        while !self.is_finished() {
            match self.rel_token(0).clone() {
                (Token::From, _) => program
                    .imports
                    .push(extract_result_if_ok!(self.parse_import())),
                (Token::Active, _) => program
                    .active
                    .push(extract_result_if_ok!(self.parse_object(true))),
                (Token::Passive, _) => program
                    .passive
                    .push(extract_result_if_ok!(self.parse_object(false))),
                (Token::EOF, _) => {
                    break;
                }
                t => {
                    return Err((
                        t,
                        "Only imports and object declarations are allowed at top level!",
                    ));
                }
            }
        }
        Ok(program)
    }

    fn parse_import(&mut self) -> Result<ImportDecl, ParseError> {
        consume_and_check!(self, Token::From);
        let module = consume_and_check_ident!(self);

        consume_and_check!(self, Token::Import);
        let mut typenames: Vec<String> = vec![];

        typenames.push(consume_and_check_type_ident!(self));

        while self.rel_token_check(0, Token::Comma) {
            self.consume_token();
            typenames.push(consume_and_check_type_ident!(self));
        }
        consume_and_check!(self, Token::Semicolon);

        Ok(ImportDecl { module, typenames })
    }

    fn parse_object(&mut self, is_active: bool) -> Result<ObjectDecl, ParseError> {
        Ok(ObjectDecl {
            is_active,
            name: String::from("Obj"),
            fields: vec![],
            methods: vec![],
        })
    }
}

pub fn parse(tokens: Vec<ScannedToken>) -> Result<Program, ParseError> {
    let mut parser = Parser::create(tokens);
    parser.parse_top_level()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::scan_tokens;

    type ParsingFunction<T> = fn(&mut Parser) -> Result<T, ParseError>;

    fn parse_helper<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) -> T {
        let tokens = scan_tokens(String::from(s));
        let mut parser = Parser::create(tokens);
        let parsed_ast = parsefn(&mut parser);

        assert!(
            parsed_ast.is_ok(),
            "Parse error: {:?}",
            parsed_ast.unwrap_err()
        );
        parsed_ast.unwrap()
    }

    #[test]
    fn simple_import() {
        assert_eq!(
            parse_helper(Parser::parse_import, "from module import Actor;"),
            ImportDecl {
                module: String::from("module"),
                typenames: vec![String::from("Actor")]
            }
        );
    }

    #[test]
    fn multiple_imports() {
        assert_eq!(
            parse_helper(
                Parser::parse_top_level,
                "from some2 import Hello, There; from two import One;"
            ),
            Program {
                imports: vec![
                    ImportDecl {
                        module: String::from("some2"),
                        typenames: vec![String::from("Hello"), String::from("There")]
                    },
                    ImportDecl {
                        module: String::from("two"),
                        typenames: vec![String::from("One")]
                    }
                ],
                passive: vec![],
                active: vec![]
            }
        );
    }

    #[test]
    fn active_object_and_fields() {
        assert_eq!(
            parse_helper(
                |p| Parser::parse_object(p, true),
                "active Actor { String name; Actor lol; }"
            ),
            ObjectDecl {
                is_active: true,
                name: String::from("Obj"),
                fields: vec![],
                methods: vec![]
            }
        );
    }
}
