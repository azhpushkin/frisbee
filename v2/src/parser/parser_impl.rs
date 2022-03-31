use super::helpers::{bin_op_from_token, unary_op_from_token};
use crate::ast::*;
use crate::scanner::*;

pub struct Parser {
    tokens: Vec<ScannedToken>,
    position: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub error_at: ScannedToken,
    pub error_msg: &'static str,
    pub expected: Option<Token>,
}
pub type ParseResult<T> = Result<T, ParseError>;

fn perr<T>(error_at: &ScannedToken, error_msg: &'static str) -> ParseResult<T> {
    Err(ParseError { error_at: error_at.clone(), error_msg, expected: None })
}

fn perr_with_expected<T>(
    error_at: &ScannedToken,
    error_msg: &'static str,
    expected: Token,
) -> ParseResult<T> {
    Err(ParseError { error_at: error_at.clone(), error_msg, expected: Some(expected) })
}

// TODO: add tests for parsing error

macro_rules! consume_and_check {
    ($self:ident, $expected:expr) => {
        match $self.consume_token() {
            (t, _) if t.eq(&$expected) => (),
            t => return perr_with_expected(t, "Unexpected token", $expected),
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
            (Token::Identifier(s), _) => s.clone(),
            t => return perr(t, "Unexpected token (expected identifier)"),
        }
    };
}

macro_rules! consume_and_check_type_ident {
    ($self:ident) => {
        match $self.consume_token() {
            (Token::TypeIdentifier(s), _) => s.clone(),
            t => return perr(t, "Unexpected token (expected identifier)"),
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

    fn consume_token(&mut self) -> &ScannedToken {
        self.position += 1;
        // TODO: check performance or smth after removing clone() everywhere in file
        self.rel_token(-1)
    }

    fn rel_token_check(&self, rel_pos: isize, token: Token) -> bool {
        match self.rel_token(rel_pos) {
            (x, _) => token.eq(x),
        }
    }

    fn is_finished(&self) -> bool {
        self.position >= self.tokens.len()
    }

    pub fn parse_top_level(&mut self) -> ParseResult<FileAst> {
        let mut file_ast = FileAst { imports: vec![], functions: vec![], types: vec![] };

        while !self.is_finished() {
            match self.rel_token(0).0 {
                Token::From => file_ast.imports.push(self.parse_import()?),
                Token::Active => file_ast.types.push(self.parse_object(true)?),
                Token::Class => file_ast.types.push(self.parse_object(false)?),
                Token::Fun => file_ast
                    .functions
                    .push(self.parse_function_definition(None)?),
                Token::EOF => {
                    break;
                }
                _ => {
                    return perr(
                        self.rel_token(0),
                        "Only imports and fun/class/active declarations are allowed at top level!",
                    );
                }
            }
        }
        Ok(file_ast)
    }

    pub fn parse_import(&mut self) -> ParseResult<ImportDecl> {
        consume_and_check!(self, Token::From);

        let mut module_path: Vec<String> = vec![consume_and_check_ident!(self)];
        while consume_if_matches_one_of!(self, [Token::Dot]) {
            module_path.push(consume_and_check_ident!(self));
        }

        consume_and_check!(self, Token::Import);
        let mut typenames: Vec<String> = vec![];
        let mut functions: Vec<String> = vec![];

        loop {
            match self.consume_token() {
                (Token::TypeIdentifier(s), _) => typenames.push(s.clone()),
                (Token::Identifier(s), _) => functions.push(s.clone()),
                t => return perr(t, "Unexpected token (expected identifier)"),
            }
            if self.rel_token_check(0, Token::Comma) {
                self.consume_token();
            } else if self.rel_token_check(0, Token::Semicolon) {
                break;
            }
        }
        consume_and_check!(self, Token::Semicolon);

        Ok(ImportDecl { module_path: ModulePath(module_path), typenames, functions })
    }

    pub fn parse_type(&mut self) -> ParseResult<Type> {
        let (token, _) = self.consume_token();
        let mut result_type = match token {
            Token::LeftSquareBrackets => {
                let item_type = self.parse_type()?;
                consume_and_check!(self, Token::RightSquareBrackets);
                Type::TypeList(Box::new(item_type))
            }
            Token::LeftParenthesis => {
                let mut tuple_items: Vec<Type> = vec![];

                until_closes!(self, Token::RightParenthesis, {
                    tuple_items.push(self.parse_type()?);
                    if self.rel_token_check(0, Token::Comma) {
                        self.consume_token();
                    }
                });

                match tuple_items.len() {
                    0 => return perr(self.rel_token(0), "Empty tuple is not allowed"),
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
                _ => Type::TypeIdent(s.clone()),
            },
            _ => {
                return perr(self.rel_token(-1), "Wrong token for type definition");
            }
        };

        while self.rel_token_check(0, Token::Question) {
            self.consume_token();
            result_type = Type::TypeMaybe(Box::new(result_type));
        }

        Ok(result_type)
    }

    pub fn parse_function_definition(
        &mut self,
        member_of: Option<&String>,
    ) -> ParseResult<FunctionDecl> {
        consume_and_check!(self, Token::Fun);
        let rettype = self.parse_type()?;

        let name: String;
        if member_of.is_some() && self.rel_token_check(0, Token::LeftParenthesis) {
            // LeftParenthesis means that this is a constuctor
            // So check if the constructor name is correct and return error if not
            name = match &rettype {
                Type::TypeIdent(s) if s != member_of.unwrap() => {
                    return perr(
                        self.rel_token(0),
                        "Wrong typename is used for constructor-like method",
                    )
                }
                Type::TypeIdent(s) => s.clone(),
                _ => return perr(self.rel_token(0), "Expected method name"),
            };
        } else {
            name = consume_and_check_ident!(self);
        }

        let mut args: Vec<TypedNamedObject> = vec![];

        consume_and_check!(self, Token::LeftParenthesis);
        until_closes!(self, Token::RightParenthesis, {
            let argtype = self.parse_type()?;
            let argname = consume_and_check_ident!(self);

            if self.rel_token_check(0, Token::Comma) {
                self.consume_token();
            }
            args.push(TypedNamedObject { typename: argtype, name: argname })
        });

        let stmts = self.parse_statements_in_curly_block(false)?;

        Ok(FunctionDecl { rettype, name, args, statements: stmts })
    }

    pub fn parse_object(&mut self, is_active: bool) -> ParseResult<ClassDecl> {
        if is_active {
            consume_and_check!(self, Token::Active);
        } else {
            consume_and_check!(self, Token::Class);
        }

        let new_object_name = consume_and_check_type_ident!(self);
        let mut fields: Vec<TypedNamedObject> = vec![];
        let mut methods: Vec<FunctionDecl> = vec![];

        consume_and_check!(self, Token::LeftCurlyBrackets);

        let is_method = |p: &mut Parser| p.rel_token_check(0, Token::Fun);
        let is_obj_end = |p: &mut Parser| p.rel_token_check(0, Token::RightCurlyBrackets);

        // Parse object fields
        while !(is_method(self) || is_obj_end(self)) {
            let typename = self.parse_type()?;
            let name = consume_and_check_ident!(self);
            consume_and_check!(self, Token::Semicolon);
            fields.push(TypedNamedObject { typename, name });
        }

        // Parse object methods
        while !is_obj_end(self) {
            let new_method = self.parse_function_definition(Some(&new_object_name))?;

            methods.push(new_method);
        }

        consume_and_check!(self, Token::RightCurlyBrackets);

        Ok(ClassDecl { is_active, name: new_object_name, fields, methods })
    }

    pub fn parse_statements_in_curly_block(
        &mut self,
        is_loop: bool,
    ) -> ParseResult<Vec<Statement>> {
        let mut statements: Vec<Statement> = vec![];
        consume_and_check!(self, Token::LeftCurlyBrackets);
        until_closes!(self, Token::RightCurlyBrackets, {
            if is_loop {
                statements.push(self.parse_statement_inside_loop()?);
            } else {
                statements.push(self.parse_statement()?);
            }
        });
        Ok(statements)
    }

    pub fn parse_if_else_stmt(&mut self) -> ParseResult<Statement> {
        consume_and_check!(self, Token::If);
        let condition = self.parse_expr()?;
        let ifbody = self.parse_statements_in_curly_block(false)?;

        let elsebody: Vec<Statement>;
        if consume_if_matches_one_of!(self, [Token::Else]) {
            elsebody = self.parse_statements_in_curly_block(false)?;
        } else {
            elsebody = vec![];
        };
        Ok(Statement::SIfElse { condition, ifbody, elsebody })
    }

    pub fn parse_while_loop_stmt(&mut self) -> ParseResult<Statement> {
        consume_and_check!(self, Token::While);
        let condition = self.parse_expr()?;
        let body = self.parse_statements_in_curly_block(true)?;
        Ok(Statement::SWhile { condition, body })
    }

    pub fn parse_foreach_loop_stmt(&mut self) -> ParseResult<Statement> {
        consume_and_check!(self, Token::Foreach);
        let itemname = consume_and_check_ident!(self);
        consume_and_check!(self, Token::In);
        let iterable = self.parse_expr()?;
        let body = self.parse_statements_in_curly_block(true)?;
        Ok(Statement::SForeach { itemname, iterable, body })
    }

    pub fn parse_var_declaration_continuation(&mut self, typedecl: Type) -> ParseResult<Statement> {
        let varname = consume_and_check_ident!(self);
        if consume_if_matches_one_of!(self, [Token::Semicolon]) {
            return Ok(Statement::SVarDecl(typedecl, varname));
        } else if consume_if_matches_one_of!(self, [Token::Equal]) {
            let value = self.parse_expr()?;
            consume_and_check!(self, Token::Semicolon);
            return Ok(Statement::SVarDeclEqual(typedecl, varname, value));
        } else {
            return perr_with_expected(
                self.rel_token(0),
                "Wrong variable declaration",
                Token::Semicolon,
            );
        }
    }

    pub fn parse_statement_inside_loop(&mut self) -> ParseResult<Statement> {
        let stmt = match self.rel_token(0).0 {
            Token::Break => Statement::SBreak,
            Token::Continue => Statement::SContinue,
            _ => return self.parse_statement(),
        };
        self.consume_token();
        consume_and_check!(self, Token::Semicolon);
        Ok(stmt)
    }

    pub fn parse_statement(&mut self) -> ParseResult<Statement> {
        let (token, _) = self.rel_token(0);
        match token {
            Token::If => return self.parse_if_else_stmt(),
            Token::While => return self.parse_while_loop_stmt(),
            Token::Foreach => return self.parse_foreach_loop_stmt(),
            Token::Return => {
                self.consume_token();
                let expr = self.parse_expr()?;
                consume_and_check!(self, Token::Semicolon);
                return Ok(Statement::SReturn(expr));
            }
            _ => (),
        }

        // First, try to consume type to see if this is type declaration
        // If Type is parsed correctly - then this must be some kind of variable declaration
        let current_pos = self.position;
        let parsed_type = self.parse_type();
        if parsed_type.is_ok() {
            return self.parse_var_declaration_continuation(parsed_type.unwrap());
        }

        // If type is not parsed, than fallback to other statement types
        // and return position to pre-type state, as parsing type might have
        // already moved it

        self.position = current_pos;

        let expr = self.parse_expr()?;

        if consume_if_matches_one_of!(self, [Token::Semicolon]) {
            // In some functional languages plain expression might be removed from AST
            // entirely as they have no effect.
            // However, in frisbee this is not true, as this is more OOP-like language
            // and expression like object method call might change its state
            return Ok(Statement::SExpr(expr));
        } else if consume_if_matches_one_of!(self, [Token::Equal]) {
            let value = self.parse_expr()?;
            consume_and_check!(self, Token::Semicolon);
            return Ok(Statement::SAssign { left: expr, right: value });
        } else if consume_if_matches_one_of!(self, [Token::Bang]) {
            let method = consume_and_check_ident!(self);
            let args = self.parse_function_call_args()?;
            consume_and_check!(self, Token::Semicolon);
            return Ok(Statement::SSendMessage { active: expr, method, args });
        } else {
            return perr_with_expected(
                self.rel_token(0),
                "Expression abruptly ended",
                Token::Semicolon,
            );
        }
    }

    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        return self.parse_expr_equality();
    }

    pub fn parse_expr_comparison(&mut self) -> ParseResult<Expr> {
        let mut res_expr = self.parse_expr_term()?;
        while consume_if_matches_one_of!(
            self,
            [Token::Greater, Token::GreaterEqual, Token::LessEqual, Token::Less]
        ) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = self.parse_expr_term()?;

            res_expr = Expr::ExprBinOp {
                left: Box::new(res_expr),
                right: Box::new(right),
                op: bin_op_from_token(op),
            };
        }

        Ok(res_expr)
    }

    pub fn parse_expr_term(&mut self) -> ParseResult<Expr> {
        let mut res_expr = self.parse_expr_factor()?;
        while consume_if_matches_one_of!(self, [Token::Minus, Token::Plus]) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = self.parse_expr_factor()?;

            res_expr = Expr::ExprBinOp {
                left: Box::new(res_expr),
                right: Box::new(right),
                op: bin_op_from_token(op),
            };
        }

        Ok(res_expr)
    }

    pub fn parse_expr_factor(&mut self) -> ParseResult<Expr> {
        let mut res_expr = self.parse_expr_unary()?;
        while consume_if_matches_one_of!(self, [Token::Star, Token::Slash]) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = self.parse_expr_unary()?;

            res_expr = Expr::ExprBinOp {
                left: Box::new(res_expr),
                right: Box::new(right),
                op: bin_op_from_token(op),
            };
        }

        Ok(res_expr)
    }

    pub fn parse_expr_equality(&mut self) -> ParseResult<Expr> {
        let mut res_expr = self.parse_expr_comparison()?;
        while consume_if_matches_one_of!(self, [Token::EqualEqual, Token::BangEqual]) {
            let (op, _) = &self.rel_token(-1).clone();
            let right = self.parse_expr_comparison()?;

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
            let operand = self.parse_method_or_field_access()?;

            let e = Expr::ExprUnaryOp { operand: Box::new(operand), op: unary_op_from_token(t) };
            return Ok(e);
        }

        return self.parse_method_or_field_access();
    }

    pub fn parse_function_call_args(&mut self) -> ParseResult<Vec<Expr>> {
        let args: Vec<Expr>;
        if self.rel_token_check(1, Token::RightParenthesis) {
            // Consume both left and right parenthesis
            self.consume_token();
            self.consume_token();
            args = vec![];
        } else {
            let args_expr = self.parse_group_or_tuple()?;
            args = match args_expr {
                Expr::ExprTupleValue(a) => a,
                e => vec![e],
            }
        }
        Ok(args)
    }

    pub fn parse_method_or_field_access(&mut self) -> ParseResult<Expr> {
        let mut res_expr = self.parse_expr_primary()?;

        while consume_if_matches_one_of!(
            self,
            [Token::Dot, Token::LeftSquareBrackets, Token::LeftParenthesis]
        ) {
            // If dot - parse field or method access
            if self.rel_token_check(-1, Token::Dot) {
                let field_or_method = consume_and_check_ident!(self);
                if self.rel_token_check(0, Token::LeftParenthesis) {
                    res_expr = Expr::ExprMethodCall {
                        object: Box::new(res_expr),
                        method: field_or_method,
                        args: self.parse_function_call_args()?,
                    };
                } else {
                    res_expr = Expr::ExprFieldAccess {
                        object: Box::new(res_expr),
                        field: field_or_method,
                    };
                }
            } else if self.rel_token_check(-1, Token::LeftSquareBrackets) {
                // otherwise - left square brackets, meaning this is list access
                let index = self.parse_expr()?;
                consume_and_check!(self, Token::RightSquareBrackets);
                res_expr = Expr::ExprListAccess { list: Box::new(res_expr), index: Box::new(index) }
            } else {
                let mut is_own_method = false;
                if matches!(res_expr, Expr::ExprOwnFieldAccess { .. }) {
                    is_own_method = true;
                }
                let cloned_identifier = match res_expr {
                    Expr::ExprIdentifier(ident) => ident.clone(),
                    Expr::ExprOwnFieldAccess { field } => field.clone(),
                    _ => return perr(self.rel_token(0), "Function call of non-function expr"),
                };
                self.position -= 1;
                let args = self.parse_function_call_args()?;
                if self.rel_token_check(0, Token::LeftParenthesis) {
                    return perr(
                        self.rel_token(0),
                        "No first-class fuctions, chained func calls disallowed",
                    );
                }
                if is_own_method {
                    res_expr = Expr::ExprOwnMethodCall { method: cloned_identifier, args };
                } else {
                    res_expr = Expr::ExprFunctionCall { function: cloned_identifier, args };
                }
            }
        }

        Ok(res_expr)
    }

    pub fn parse_group_or_tuple(&mut self) -> ParseResult<Expr> {
        consume_and_check!(self, Token::LeftParenthesis);
        let mut result_expr = self.parse_expr()?;

        // If comma - then this is not just grouping, but a tuple
        if self.rel_token_check(0, Token::Comma) {
            let mut tuple_exprs: Vec<Expr> = vec![result_expr];

            // For the next items, trailing comma is allowed, so expr after comma is optional
            while consume_if_matches_one_of!(self, [Token::Comma]) {
                if self.rel_token_check(0, Token::RightParenthesis) {
                    break;
                }
                tuple_exprs.push(self.parse_expr()?);
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
            list_items.push(self.parse_expr()?);
            consume_if_matches_one_of!(self, [Token::Comma]);
        });

        return Ok(Expr::ExprListValue(list_items));
    }

    fn parse_new_class_instance_expr(&mut self) -> ParseResult<Expr> {
        let typename = consume_and_check_type_ident!(self);
        let args = self.parse_function_call_args()?;
        Ok(Expr::ExprNewClassInstance { typename, args })
    }

    fn parse_spawn_active_expr(&mut self) -> ParseResult<Expr> {
        consume_and_check!(self, Token::Spawn);
        let typename = consume_and_check_type_ident!(self);
        let args = self.parse_function_call_args()?;
        Ok(Expr::ExprSpawnActive { typename, args })
    }

    pub fn parse_expr_primary(&mut self) -> ParseResult<Expr> {
        let expr = match &self.rel_token(0).0 {
            Token::This => Expr::ExprThis,
            Token::Float(f) => Expr::ExprFloat(f.clone()),
            Token::Integer(i) => Expr::ExprInt(i.clone()),
            Token::String(s) => Expr::ExprString(s.clone()),
            Token::Nil => Expr::ExprNil,
            Token::True => Expr::ExprBool(true),
            Token::False => Expr::ExprBool(false),
            Token::Identifier(i) => Expr::ExprIdentifier(i.clone()),
            Token::OwnIdentifier(f) => Expr::ExprOwnFieldAccess { field: f.clone() },
            Token::LeftParenthesis => return self.parse_group_or_tuple(),
            Token::LeftSquareBrackets => return self.parse_list_literal(),
            Token::TypeIdentifier(_) => return self.parse_new_class_instance_expr(),
            Token::Spawn => return self.parse_spawn_active_expr(),
            t => {
                return perr_with_expected(self.rel_token(0), "Unexpected expression", t.clone());
            }
        };

        self.consume_token();

        Ok(expr)
    }
}
