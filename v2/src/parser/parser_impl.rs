use crate::ast::*;
use crate::tokens::*;

pub struct Parser {
    tokens: Vec<ScannedToken>,
    position: usize,
}

pub type ParseError = (ScannedToken, &'static str, Option<Token>);
pub type ParseResult<T> = Result<T, ParseError>;
// TODO: add tests for parsing error

macro_rules! extract_result_if_ok {
    ($parse_result:expr) => {
        match $parse_result {
            Ok(res) => res,
            // Re-wrap pf parsing error is required to coerce type
            // from Result<T, ParseError> to Result<Program, ParseError>
            Err(t) => return Err(t),
        }
    };
}

macro_rules! consume_and_check {
    ($self:ident, $expected:expr) => {
        match $self.consume_token() {
            (t, _) if t.eq(&$expected) => (),
            t => return Err((t, "Unexpected token", Some($expected))),
        }
    };
}

macro_rules! consume_and_check_ident {
    ($self:ident) => {
        match $self.consume_token() {
            (Token::Identifier(s), _) => s,
            t => return Err((t, "Unexpected token (expected identifier)", None)),
        }
    };
}

macro_rules! consume_and_check_type_ident {
    ($self:ident) => {
        match $self.consume_token() {
            (Token::TypeIdentifier(s), _) => s,
            t => return Err((t, "Unexpected token (expected identifier)", None)),
        }
    };
}

macro_rules! until_closes {
    ($self:ident, $right_limiter:expr, $code:block) => {
        while !$self.rel_token_check(0, $right_limiter) $code
        consume_and_check!($self, $right_limiter);
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
                        None,
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
        let mut result_type = match token {
            Token::LeftSquareBrackets => {
                let item_type = extract_result_if_ok!(self.parse_type());
                consume_and_check!(self, Token::RightSquareBrackets);
                Type::TypeList(Box::new(item_type))
            }
            Token::LeftParenthesis => {
                let mut tuple_items: Vec<Type> = vec![];

                until_closes!(self, Token::RightParenthesis, {
                    tuple_items.push(extract_result_if_ok!(self.parse_type()));
                    if self.rel_token_check(0, Token::Comma) {
                        self.consume_token();
                    }
                });

                match tuple_items.len() {
                    0 => return Err(((token, pos), "Empty tuple is not allowed", None)),
                    1 => tuple_items.pop().unwrap(),
                    _ => Type::TypeTuple(tuple_items),
                }
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
                return Err((
                    (token.clone(), pos),
                    "Wrong token for type definition",
                    Some(token),
                ));
            }
        };

        while self.rel_token_check(0, Token::Question) {
            self.consume_token();
            result_type = Type::TypeMaybe(Box::new(result_type));
        }

        Ok(result_type)
    }

    pub fn parse_object(&mut self, is_active: bool) -> ParseResult<ObjectDecl> {
        if is_active {
            consume_and_check!(self, Token::Active);
        } else {
            consume_and_check!(self, Token::Passive);
        }

        let name = consume_and_check_type_ident!(self);
        let mut fields: Vec<TypedNamedObject> = vec![];
        let mut methods: Vec<MethodDecl> = vec![];

        consume_and_check!(self, Token::LeftCurlyBrackets);

        let is_method = |p: &mut Parser| p.rel_token_check(0, Token::Fun);
        let is_obj_end = |p: &mut Parser| p.rel_token_check(0, Token::RightCurlyBrackets);

        // Parse object fields
        while !(is_method(self) || is_obj_end(self)) {
            let typename = extract_result_if_ok!(self.parse_type());
            let name = consume_and_check_ident!(self);
            consume_and_check!(self, Token::Semicolon);
            fields.push(TypedNamedObject { typename, name });
        }

        // Parse object methods
        while !is_obj_end(self) {
            consume_and_check!(self, Token::Fun);
            let rettype = extract_result_if_ok!(self.parse_type());
            let name = consume_and_check_ident!(self);
            let mut args: Vec<TypedNamedObject> = vec![];
            let mut stmts: Vec<Statement> = vec![];

            consume_and_check!(self, Token::LeftParenthesis);
            until_closes!(self, Token::RightParenthesis, {
                let argtype = extract_result_if_ok!(self.parse_type());
                let argname = consume_and_check_ident!(self);

                if self.rel_token_check(0, Token::Comma) {
                    self.consume_token();
                }
                args.push(TypedNamedObject { typename: argtype, name: argname });
            });

            consume_and_check!(self, Token::LeftCurlyBrackets);
            consume_and_check!(self, Token::RightCurlyBrackets); // TODO; remote after stmt done
            // until_closes!(self, Token::RightCurlyBrackets, {
            //     // stmts.push(extract_result_if_ok!(self.parse_statement()));
            // });
            methods.push(MethodDecl { rettype, name, args, statements: stmts });
        }

        consume_and_check!(self, Token::RightCurlyBrackets);

        Ok(ObjectDecl { is_active, name, fields, methods })
    }

    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
        Ok(Statement::SExpr(Expr::ExprThis))
    }

    pub fn parse_expression(&mut self) -> ParseResult<Expr> {
        Ok(Expr::ExprThis)
    }
}
