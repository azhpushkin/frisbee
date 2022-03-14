use super::helpers::{bin_op_from_token, unary_op_from_token};
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

macro_rules! consume_if_matches_one_of {
    ($self:ident, $expected_arr:expr) => {{
        match $self.rel_token(0) {
            (t, _) if $expected_arr.contains(t) => {
                $self.consume_token();
                true
            }
            _ => false,
        }
    }};
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
            until_closes!(self, Token::RightCurlyBrackets, {
                stmts.push(extract_result_if_ok!(self.parse_statement()));
            });

            // consume_and_check!(self, Token::RightCurlyBrackets); // TODO; remote after stmt done
            // until_closes!(self, Token::RightCurlyBrackets, {
            //     // stmts.push(extract_result_if_ok!(self.parse_statement()));
            // });
            methods.push(MethodDecl { rettype, name, args, statements: stmts });
        }

        consume_and_check!(self, Token::RightCurlyBrackets);

        Ok(ObjectDecl { is_active, name, fields, methods })
    }

    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
        let (token, _) = self.rel_token(0);
        match token {
            &Token::If => panic!("If is not done!"),
            &Token::While => panic!("While is not done!"),
            &Token::Return => panic!("Return is not done!"),
            &Token::Let => panic!("VarDecl is not done!"),
            _ => {
                let expr = extract_result_if_ok!(self.parse_expr());
                if consume_if_matches_one_of!(self, [Token::Bang, Token::Equal, Token::Semicolon]) {
                    let (prev, _) = self.rel_token(-1);
                    match prev {
                        &Token::Semicolon => return Ok(Statement::SExpr(expr)),
                        _ => panic!("NOT DONE!"),
                    }
                } else {
                    panic!("Unexpected token after expr: {:?}", self.rel_token(0))
                }
            }
        }
    }

    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        return self.parse_expr_equality();
    }

    pub fn parse_expr_comparison(&mut self) -> ParseResult<Expr> {
        let mut res_expr = extract_result_if_ok!(self.parse_expr_term());
        while consume_if_matches_one_of!(
            self,
            [Token::Greater, Token::GreaterEqual, Token::LessEqual, Token::Less]
        ) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = extract_result_if_ok!(self.parse_expr_term());

            res_expr = Expr::ExprBinOp {
                left: Box::new(res_expr),
                right: Box::new(right),
                op: bin_op_from_token(op),
            };
        }

        Ok(res_expr)
    }

    pub fn parse_expr_term(&mut self) -> ParseResult<Expr> {
        let mut res_expr = extract_result_if_ok!(self.parse_expr_factor());
        while consume_if_matches_one_of!(self, [Token::Minus, Token::Plus]) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = extract_result_if_ok!(self.parse_expr_factor());

            res_expr = Expr::ExprBinOp {
                left: Box::new(res_expr),
                right: Box::new(right),
                op: bin_op_from_token(op),
            };
        }

        Ok(res_expr)
    }

    pub fn parse_expr_factor(&mut self) -> ParseResult<Expr> {
        let mut res_expr = extract_result_if_ok!(self.parse_expr_unary());
        while consume_if_matches_one_of!(self, [Token::Star, Token::Slash]) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = extract_result_if_ok!(self.parse_expr_unary());

            res_expr = Expr::ExprBinOp {
                left: Box::new(res_expr),
                right: Box::new(right),
                op: bin_op_from_token(op),
            };
        }

        Ok(res_expr)
    }

    pub fn parse_expr_equality(&mut self) -> ParseResult<Expr> {
        let mut res_expr = extract_result_if_ok!(self.parse_expr_comparison());
        while consume_if_matches_one_of!(self, [Token::EqualEqual, Token::BangEqual]) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = extract_result_if_ok!(self.parse_expr_comparison());

            res_expr = Expr::ExprBinOp {
                left: Box::new(res_expr),
                right: Box::new(right),
                op: bin_op_from_token(op),
            };
        }

        Ok(res_expr)
    }

    pub fn parse_expr_unary(&mut self) -> ParseResult<Expr> {
        if consume_if_matches_one_of!(self, [Token::Minus, Token::Not]) {
            let (t, _) = &self.rel_token(-1).clone();
            let operand = extract_result_if_ok!(self.parse_method_or_field_access());

            let e = Expr::ExprUnaryOp { operand: Box::new(operand), op: unary_op_from_token(t) };
            return Ok(e);
        }

        return self.parse_method_or_field_access();
    }

    pub fn parse_method_args(&mut self) -> ParseResult<Vec<Expr>> {
        let args: Vec<Expr>;
        if self.rel_token_check(1, Token::RightParenthesis) {
            // Consume both left and right parenthesis
            self.consume_token();
            self.consume_token();
            args = vec![];
        } else {
            let args_expr = extract_result_if_ok!(self.parse_group_or_tuple());
            args = match args_expr {
                Expr::ExprTupleValue(a) => a,
                e => vec![e],
            }
        }
        Ok(args)
    }

    pub fn parse_method_or_field_access(&mut self) -> ParseResult<Expr> {
        let mut res_expr = extract_result_if_ok!(self.parse_expr_primary());

        while consume_if_matches_one_of!(self, [Token::Dot, Token::LeftSquareBrackets]) {
            // If dot - parse field or method access
            if self.rel_token_check(-1, Token::Dot) {
                let field_or_method = consume_and_check_ident!(self);
                if self.rel_token_check(0, Token::LeftParenthesis) {
                    res_expr = Expr::ExprMethodCall {
                        object: Box::new(res_expr),
                        method: field_or_method,
                        args: extract_result_if_ok!(self.parse_method_args()),
                    };
                } else {
                    res_expr = Expr::ExprFieldAccess {
                        object: Box::new(res_expr),
                        field: field_or_method,
                    };
                }
            } else {
                // otherwise - left square brackets, meaning this is list access
                let index = extract_result_if_ok!(self.parse_expr());
                consume_and_check!(self, Token::RightSquareBrackets);
                res_expr = Expr::ExprListAccess { list: Box::new(res_expr), index: Box::new(index) }
            }
        }

        Ok(res_expr)
    }

    pub fn parse_group_or_tuple(&mut self) -> ParseResult<Expr> {
        consume_and_check!(self, Token::LeftParenthesis);
        let mut result_expr = extract_result_if_ok!(self.parse_expr());

        // If comma - then this is not just grouping, but a tuple
        if self.rel_token_check(0, Token::Comma) {
            let mut tuple_exprs: Vec<Expr> = vec![result_expr];

            // For the next items, trailing comma is allowed, so expr after comma is optional
            while consume_if_matches_one_of!(self, [Token::Comma]) {
                if self.rel_token_check(0, Token::RightParenthesis) {
                    break;
                }
                tuple_exprs.push(extract_result_if_ok!(self.parse_expr()));
            }

            if tuple_exprs.len() == 1 {
                result_expr = tuple_exprs.pop().unwrap();
            } else {
                result_expr = Expr::ExprTupleValue(tuple_exprs);
            }
        }

        consume_and_check!(self, Token::RightParenthesis);
        Ok(result_expr)
    }

    pub fn parse_list_literal(&mut self) -> ParseResult<Expr> {
        consume_and_check!(self, Token::LeftSquareBrackets);
        let mut list_items: Vec<Expr> = vec![];

        until_closes!(self, Token::RightSquareBrackets, {
            list_items.push(extract_result_if_ok!(self.parse_expr()));
            consume_if_matches_one_of!(self, [Token::Comma]);
        });

        return Ok(Expr::ExprListValue(list_items));
    }

    pub fn parse_expr_primary(&mut self) -> ParseResult<Expr> {
        let (token, pos) = self.rel_token(0);

        let expr = match token {
            Token::This => Expr::ExprThis,
            Token::Float(f) => Expr::ExprFloat(f.clone()),
            Token::Integer(i) => Expr::ExprInt(i.clone()),
            Token::String(s) => Expr::ExprString(s.clone()),
            Token::Nil => Expr::ExprNil,
            Token::True => Expr::ExprBool(true),
            Token::False => Expr::ExprBool(false),
            Token::Identifier(i) => Expr::ExprIdentifier(i.clone()),
            Token::LeftParenthesis => return self.parse_group_or_tuple(),
            Token::LeftSquareBrackets => return self.parse_list_literal(),
            _ => {
                return Err((
                    (token.clone(), pos.clone()),
                    "Unexpected expression",
                    Some(token.clone()),
                ))
            }
        };

        self.consume_token();

        Ok(expr)
    }
}
