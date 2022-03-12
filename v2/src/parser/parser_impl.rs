use std::result;

use crate::ast::*;
use crate::tokens::*;

pub struct Parser {
    tokens: Vec<ScannedToken>,
    position: usize,
}

pub type ParseError = (ScannedToken, &'static str);
pub type ParseResult<T> = Result<T, ParseError>;
// TODO: add expected token
// TODO: add tests for parsing error

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
    pub fn create(tokens: Vec<ScannedToken>) -> Parser {
        Parser { tokens, position: 0 }
    }

    fn rel_token(&self, rel_pos: isize) -> &ScannedToken {
        let pos = if rel_pos < 0 {
            self.position - (rel_pos.abs() as usize)
        } else {
            self.position + (rel_pos as usize)
        };

        match self.tokens.get(pos) {
            Some(x) => x,
            None => &(Token::EOF, 0), // 0 here is strange but IDK what else
        }
    }

    fn consume_token(&mut self) -> ScannedToken {
        self.position += 1;
        // TODO: check performance or smth after removing clone() everywhere in file
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

    pub fn parse_top_level(&mut self) -> ParseResult<Program> {
        let mut program = Program { imports: vec![], passive: vec![], active: vec![] };

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

    pub fn parse_import(&mut self) -> ParseResult<ImportDecl> {
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

    pub fn parse_type(&mut self) -> ParseResult<Type> {
        let (token, pos) = self.consume_token();
        let result_type = match token {
            Token::LeftSquareBrackets => {
                let item_type = extract_result_if_ok!(self.parse_type());
                consume_and_check!(self, Token::RightSquareBrackets);
                Type::TypeList(Box::new(item_type))
            }
            Token::TypeIdentifier(s) => match s.as_str() {
                "Int" => Type::TypeInt,
                "Float" => Type::TypeFloat,
                "Nil" => Type::TypeNil,
                "Bool" => Type::TypeBool,
                "String" => Type::TypeString,
                _ => Type::TypeIdent(s),
            },
            _ => {
                return Err(((token, pos), "Wrong token for type definition"));
            }
        };

        Ok(result_type)
    }

    pub fn parse_object(&mut self, is_active: bool) -> ParseResult<ObjectDecl> {
        consume_and_check!(self, Token::Active);

        let name = consume_and_check_type_ident!(self);
        let fields: Vec<TypedNamedObject> = vec![];
        let methods: Vec<MethodDecl> = vec![];

        consume_and_check!(self, Token::LeftCurlyBrackets);

        consume_and_check!(self, Token::RightCurlyBrackets);

        Ok(ObjectDecl { is_active, name, fields, methods })
    }
}
