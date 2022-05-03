use super::scanner::*;
use crate::ast::parsed::*;
use crate::types::ParsedType;

use super::helpers::{bin_op_from_token, unary_op_from_token};

pub struct Parser<'a> {
    tokens: &'a Vec<ScannedToken>,
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
            t if t.eq(&$expected) => (),
            _ => return perr_with_expected($self.full_token(-1), "Unexpected token", $expected),
        }
    };
}

macro_rules! consume_if_matches_one_of {
    ($self:ident, $expected_arr:expr) => {{
        match $self.rel_token(0) {
            t if $expected_arr.contains(t) => {
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
            Token::Identifier(s) => s.clone(),
            _ => {
                return perr(
                    $self.full_token(-1),
                    "Unexpected token (expected identifier)",
                )
            }
        }
    };
}

macro_rules! consume_and_check_type_ident {
    ($self:ident) => {
        match $self.consume_token() {
            Token::TypeIdentifier(s) => s.clone(),
            _ => {
                return perr(
                    $self.full_token(-1),
                    "Unexpected token (expected identifier)",
                )
            }
        }
    };
}

macro_rules! until_closes {
    ($self:ident, $right_limiter:expr, $code:block) => {
        while !$self.rel_token_check(0, $right_limiter) $code
        consume_and_check!($self, $right_limiter);
    };
}

impl<'a> Parser<'a> {
    pub fn create(tokens: &'a Vec<ScannedToken>) -> Parser<'a> {
        Parser { tokens, position: 0 }
    }

    fn full_token(&self, rel_pos: isize) -> &ScannedToken {
        let pos = if rel_pos < 0 {
            self.position - (rel_pos.abs() as usize)
        } else {
            self.position + (rel_pos as usize)
        };

        match self.tokens.get(pos) {
            Some(x) => x,
            None => &ScannedToken { token: Token::EOF, first: 0, last: 0 }, // 0 here is strange but IDK what else
        }
    }

    fn rel_token(&self, rel_pos: isize) -> &Token {
        &self.full_token(rel_pos).token
    }

    fn expr_with_pos(&self, expr: Expr, start: usize, end: usize) -> ParseResult<ExprWithPos> {
        Ok(ExprWithPos {
            expr,
            pos_first: self.tokens[start].first,
            pos_last: self.tokens[end].last,
        })
    }

    fn stmt_with_pos(&self, stmt: Statement, pos: usize) -> ParseResult<StatementWithPos> {
        Ok(StatementWithPos { statement: stmt, pos: self.tokens[pos].first })
    }

    fn consume_token(&mut self) -> &Token {
        self.position += 1;
        // TODO: check performance or smth after removing clone() everywhere in file
        self.rel_token(-1)
    }

    fn rel_token_check(&self, rel_pos: isize, token: Token) -> bool {
        self.rel_token(rel_pos) == &token
    }

    fn is_finished(&self) -> bool {
        self.position >= self.tokens.len()
    }

    pub fn parse_top_level(&mut self) -> ParseResult<FileAst> {
        let mut file_ast = FileAst { imports: vec![], functions: vec![], types: vec![] };

        while !self.is_finished() {
            match self.rel_token(0) {
                Token::From => file_ast.imports.push(self.parse_import()?),
                Token::Active => file_ast.types.push(self.parse_object(true)?),
                Token::Class => file_ast.types.push(self.parse_object(false)?),
                Token::Fun => file_ast.functions.push(self.parse_function_definition(None)?),
                Token::EOF => {
                    break;
                }
                _ => {
                    return perr(
                        self.full_token(0),
                        "Only imports and fun/class/active declarations are allowed at top level!",
                    );
                }
            }
        }
        Ok(file_ast)
    }

    pub fn parse_import(&mut self) -> ParseResult<ImportDecl> {
        let start_pos = self.tokens[self.position].first;
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
                Token::TypeIdentifier(s) => typenames.push(s.clone()),
                Token::Identifier(s) => functions.push(s.clone()),
                _ => {
                    return perr(
                        self.full_token(-1),
                        "Unexpected token (expected identifier)",
                    )
                }
            }
            if self.rel_token_check(0, Token::Comma) {
                self.consume_token();
            } else if self.rel_token_check(0, Token::Semicolon) {
                break;
            }
        }
        consume_and_check!(self, Token::Semicolon);

        Ok(ImportDecl { pos: start_pos, module_path, typenames, functions })
    }

    pub fn parse_type(&mut self) -> ParseResult<ParsedType> {
        let mut result_type = match self.consume_token() {
            Token::LeftSquareBrackets => {
                let item_type = self.parse_type()?;
                consume_and_check!(self, Token::RightSquareBrackets);
                ParsedType::List(Box::new(item_type))
            }
            Token::LeftParenthesis => {
                let mut tuple_items: Vec<ParsedType> = vec![];

                until_closes!(self, Token::RightParenthesis, {
                    tuple_items.push(self.parse_type()?);
                    if self.rel_token_check(0, Token::Comma) {
                        self.consume_token();
                    }
                });

                match tuple_items.len() {
                    0 => return perr(self.full_token(0), "Empty tuple is not allowed"),
                    1 => tuple_items.pop().unwrap(),
                    _ => ParsedType::Tuple(tuple_items),
                }
            }
            Token::TypeIdentifier(s) => match s.as_str() {
                "Int" => ParsedType::Int,
                "Float" => ParsedType::Float,
                "Bool" => ParsedType::Bool,
                "String" => ParsedType::String,
                _ => ParsedType::Custom(s.clone()),
            },
            _ => {
                return perr(self.full_token(-1), "Wrong token for type definition");
            }
        };

        while self.rel_token_check(0, Token::Question) {
            self.consume_token();
            result_type = ParsedType::Maybe(Box::new(result_type));
        }

        Ok(result_type)
    }

    fn parse_function_definition(
        &mut self,
        member_of: Option<&String>,
    ) -> ParseResult<FunctionDecl> {
        let declaration_start = self.tokens[self.position].first;
        consume_and_check!(self, Token::Fun);
        let rettype = match self.rel_token(0) {
            Token::Void => {
                self.consume_token();
                None
            }
            _ => Some(self.parse_type()?),
        };

        let name: String = if self.rel_token_check(0, Token::LeftParenthesis) {
            // LeftParenthesis means that this is a constuctor
            // So check if the constructor name is correct and return error if not
            if let Some(member_of) = member_of {
                match &rettype {
                    Some(ParsedType::Custom(s)) if s == member_of => s.clone(),
                    _ => return perr(self.full_token(0), "Method name is missing"),
                }
            } else {
                return perr(self.full_token(0), "Function is missing name");
            }
        } else {
            consume_and_check_ident!(self)
        };

        let mut args: Vec<TypedItem> = vec![];

        consume_and_check!(self, Token::LeftParenthesis);
        until_closes!(self, Token::RightParenthesis, {
            let argtype = self.parse_type()?;
            let argname = consume_and_check_ident!(self);

            if self.rel_token_check(0, Token::Comma) {
                self.consume_token();
            }
            args.push(TypedItem { typename: argtype, name: argname })
        });

        let stmts = self.parse_statements_in_curly_block()?;

        Ok(FunctionDecl { pos: declaration_start, rettype, name, args, statements: stmts })
    }

    pub fn parse_object(&mut self, is_active: bool) -> ParseResult<ClassDecl> {
        let declaration_start = self.tokens[self.position].first;

        if is_active {
            consume_and_check!(self, Token::Active);
        } else {
            consume_and_check!(self, Token::Class);
        }

        let new_object_name = consume_and_check_type_ident!(self);
        let mut fields: Vec<TypedItem> = vec![];
        let mut methods: Vec<FunctionDecl> = vec![];

        consume_and_check!(self, Token::LeftCurlyBrackets);

        let is_method = |p: &mut Parser| p.rel_token_check(0, Token::Fun);
        let is_obj_end = |p: &mut Parser| p.rel_token_check(0, Token::RightCurlyBrackets);

        // Parse object fields
        while !(is_method(self) || is_obj_end(self)) {
            let typename = self.parse_type()?;
            let name = consume_and_check_ident!(self);
            consume_and_check!(self, Token::Semicolon);
            fields.push(TypedItem { typename, name });
        }

        // Parse object methods
        while !is_obj_end(self) {
            let new_method = self.parse_function_definition(Some(&new_object_name))?;

            methods.push(new_method);
        }

        consume_and_check!(self, Token::RightCurlyBrackets);

        Ok(ClassDecl {
            pos: declaration_start,
            is_active,
            name: new_object_name,
            fields,
            methods,
        })
    }

    fn parse_statements_in_curly_block(&mut self) -> ParseResult<Vec<StatementWithPos>> {
        let mut statements: Vec<StatementWithPos> = vec![];
        consume_and_check!(self, Token::LeftCurlyBrackets);
        until_closes!(self, Token::RightCurlyBrackets, {
            statements.push(self.parse_statement()?);
        });
        Ok(statements)
    }

    fn parse_if_else_stmt(&mut self) -> ParseResult<StatementWithPos> {
        let start = self.position;
        consume_and_check!(self, Token::If);
        let condition = self.parse_expr()?;

        let if_body = self.parse_statements_in_curly_block()?;
        let mut elif_bodies = vec![];
        let mut else_body = vec![];

        while consume_if_matches_one_of!(self, [Token::Elif]) {
            let elif_condition = self.parse_expr()?;
            let elif_body = self.parse_statements_in_curly_block()?;
            elif_bodies.push((elif_condition, elif_body));
        }

        if consume_if_matches_one_of!(self, [Token::Else]) {
            else_body = self.parse_statements_in_curly_block()?;
        }
        self.stmt_with_pos(
            Statement::IfElse { condition, if_body, elif_bodies, else_body },
            start,
        )
    }

    fn parse_while_loop_stmt(&mut self) -> ParseResult<StatementWithPos> {
        let start = self.position;
        consume_and_check!(self, Token::While);
        let condition = self.parse_expr()?;
        let body = self.parse_statements_in_curly_block()?;
        self.stmt_with_pos(Statement::While { condition, body }, start)
    }

    fn parse_foreach_loop_stmt(&mut self) -> ParseResult<StatementWithPos> {
        let start = self.position;
        consume_and_check!(self, Token::Foreach);
        let item_name = consume_and_check_ident!(self);
        consume_and_check!(self, Token::In);
        let iterable = self.parse_expr()?;
        let body = self.parse_statements_in_curly_block()?;
        self.stmt_with_pos(Statement::Foreach { item_name, iterable, body }, start)
    }

    fn parse_var_declaration_continuation(
        &mut self,
        typedecl: ParsedType,
        start: usize,
    ) -> ParseResult<StatementWithPos> {
        let varname = consume_and_check_ident!(self);
        if consume_if_matches_one_of!(self, [Token::Semicolon]) {
            self.stmt_with_pos(Statement::VarDecl(typedecl, varname), start)
        } else if consume_if_matches_one_of!(self, [Token::Equal]) {
            let value = self.parse_expr()?;
            consume_and_check!(self, Token::Semicolon);
            self.stmt_with_pos(
                Statement::VarDeclWithAssign(typedecl, varname, value),
                start,
            )
        } else {
            perr(
                self.full_token(0),
                "Expected assignment or semicolon to finish declaration",
            )
        }
    }

    pub fn parse_statement(&mut self) -> ParseResult<StatementWithPos> {
        let start = self.position;
        match self.rel_token(0) {
            Token::Break => {
                self.consume_token();
                consume_and_check!(self, Token::Semicolon);
                return self.stmt_with_pos(Statement::Break, start);
            }
            Token::Continue => {
                self.consume_token();
                consume_and_check!(self, Token::Semicolon);
                return self.stmt_with_pos(Statement::Continue, start);
            }
            Token::If => return self.parse_if_else_stmt(),
            Token::While => return self.parse_while_loop_stmt(),
            Token::Foreach => return self.parse_foreach_loop_stmt(),
            Token::Return => {
                self.consume_token();
                if consume_if_matches_one_of!(self, [Token::Semicolon]) {
                    return self.stmt_with_pos(Statement::Return(None), start);
                }
                let expr = self.parse_expr()?;
                consume_and_check!(self, Token::Semicolon);
                return self.stmt_with_pos(Statement::Return(Some(expr)), start);
            }
            _ => (),
        }

        // First, try to consume type to see if this is type declaration
        // If Type is parsed correctly - then this must be some kind of variable declaration
        let current_pos = self.position;
        let parsed_type = self.parse_type();
        if let Ok(parsed_type) = parsed_type {
            return self.parse_var_declaration_continuation(parsed_type, start);
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
            self.stmt_with_pos(Statement::Expr(expr), start)
        } else if consume_if_matches_one_of!(self, [Token::Equal]) {
            let value = self.parse_expr()?;
            consume_and_check!(self, Token::Semicolon);
            self.stmt_with_pos(Statement::Assign { left: expr, right: value }, start)
        } else if consume_if_matches_one_of!(self, [Token::Bang]) {
            let method = consume_and_check_ident!(self);
            let args = self.parse_function_call_args()?;
            consume_and_check!(self, Token::Semicolon);
            self.stmt_with_pos(Statement::SendMessage { active: expr, method, args }, start)
        } else {
            perr_with_expected(
                self.full_token(0),
                "Expression abruptly ended",
                Token::Semicolon,
            )
        }
    }

    pub fn parse_expr(&mut self) -> ParseResult<ExprWithPos> {
        self.parse_maybe_operators()
    }

    fn parse_expr_comparison(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let mut res_expr = self.parse_expr_plus_minus()?;
        while consume_if_matches_one_of!(
            self,
            [Token::Greater, Token::GreaterEqual, Token::LessEqual, Token::Less]
        ) {
            let op = bin_op_from_token(self.rel_token(-1));
            let right = self.parse_expr_plus_minus()?;

            let inner = Expr::BinOp { left: Box::new(res_expr), right: Box::new(right), op };
            res_expr = self.expr_with_pos(inner, start, self.position - 1)?;
        }

        Ok(res_expr)
    }

    fn parse_expr_plus_minus(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let mut res_expr = self.parse_expr_mul_div()?;
        while consume_if_matches_one_of!(self, [Token::Minus, Token::Plus]) {
            let op = bin_op_from_token(self.rel_token(-1));
            let right = self.parse_expr_mul_div()?;

            let inner = Expr::BinOp { left: Box::new(res_expr), right: Box::new(right), op };
            res_expr = self.expr_with_pos(inner, start, self.position - 1)?;
        }

        Ok(res_expr)
    }

    fn parse_expr_mul_div(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let mut res_expr = self.parse_expr_unary()?;
        while consume_if_matches_one_of!(self, [Token::Star, Token::Slash]) {
            let op = bin_op_from_token(self.rel_token(-1));
            let right = self.parse_expr_unary()?;

            let inner = Expr::BinOp { left: Box::new(res_expr), right: Box::new(right), op };
            res_expr = self.expr_with_pos(inner, start, self.position - 1)?;
        }

        Ok(res_expr)
    }

    fn parse_maybe_operators(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let mut res_expr = self.parse_expr_bool_operators()?;
        while consume_if_matches_one_of!(self, [Token::QuestionElvis, Token::QuestionDot]) {
            let inner;
            if self.rel_token_check(-1, Token::QuestionElvis) {
                let right = self.parse_expr_bool_operators()?;
                inner = Expr::BinOp {
                    left: Box::new(res_expr),
                    right: Box::new(right),
                    op: BinaryOp::Elvis,
                };
                
            } else {
                let field_or_method = consume_and_check_ident!(self);
                inner = Expr::MaybeMethodCall {
                    object: Box::new(res_expr),
                    method: field_or_method,
                    args: self.parse_function_call_args()?,
                };
            }
            res_expr = self.expr_with_pos(inner, start, self.position - 1)?;
        }

        Ok(res_expr)
    }

    fn parse_expr_bool_operators(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let mut res_expr = self.parse_expr_equality()?;
        while consume_if_matches_one_of!(self, [Token::And, Token::Or]) {
            let op = bin_op_from_token(self.rel_token(-1));
            let right = self.parse_expr_equality()?;

            let inner = Expr::BinOp { left: Box::new(res_expr), right: Box::new(right), op };
            res_expr = self.expr_with_pos(inner, start, self.position - 1)?;
        }

        Ok(res_expr)
    }

    fn parse_expr_equality(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let mut res_expr = self.parse_expr_comparison()?;
        while consume_if_matches_one_of!(self, [Token::EqualEqual, Token::BangEqual]) {
            let op = bin_op_from_token(self.rel_token(-1));
            let right = self.parse_expr_comparison()?;

            let inner = Expr::BinOp { left: Box::new(res_expr), right: Box::new(right), op };
            res_expr = self.expr_with_pos(inner, start, self.position - 1)?;
        }

        Ok(res_expr)
    }

    fn parse_expr_unary(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        if consume_if_matches_one_of!(self, [Token::Minus, Token::Not]) {
            let op = unary_op_from_token(self.rel_token(-1));
            let operand = self.parse_method_or_field_access()?;

            let inner = Expr::UnaryOp { operand: Box::new(operand), op };
            return self.expr_with_pos(inner, start, self.position - 1);
        }

        self.parse_method_or_field_access()
    }

    fn parse_function_call_args(&mut self) -> ParseResult<Vec<ExprWithPos>> {
        if self.rel_token_check(1, Token::RightParenthesis) {
            // Consume both left and right parenthesis
            self.consume_token();
            self.consume_token();
            Ok(vec![])
        } else {
            // There is at least one expr here after check above
            consume_and_check!(self, Token::LeftParenthesis);
            let mut args_expr = vec![self.parse_expr()?];

            while consume_if_matches_one_of!(self, [Token::Comma]) {
                if self.rel_token_check(0, Token::RightParenthesis) {
                    break;
                }
                args_expr.push(self.parse_expr()?);
            }
            consume_and_check!(self, Token::RightParenthesis);
            Ok(args_expr)
        }
    }

    fn parse_method_or_field_access(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let mut res_expr = self.parse_expr_primary()?;

        while consume_if_matches_one_of!(
            self,
            [Token::Dot, Token::LeftSquareBrackets, Token::LeftParenthesis]
        ) {
            let inner: Expr;
            let boxed_res = Box::new(res_expr);

            if self.rel_token_check(-1, Token::Dot) {
                // First, check for dot to see if this is method or field access
                let field_or_method = consume_and_check_ident!(self);
                if self.rel_token_check(0, Token::LeftParenthesis) {
                    inner = Expr::MethodCall {
                        object: boxed_res,
                        method: field_or_method,
                        args: self.parse_function_call_args()?,
                    };
                } else {
                    inner = Expr::FieldAccess { object: boxed_res, field: field_or_method };
                }
            } else if self.rel_token_check(-1, Token::LeftSquareBrackets) {
                // Then, check for left square brackets, which indicates list or tuple access by index
                let index = self.parse_expr()?;
                consume_and_check!(self, Token::RightSquareBrackets);
                inner = Expr::ListAccess { list: boxed_res, index: Box::new(index) };
            } else {
                // Lastly, check if this is a function call
                //  If called object is Identifier - than this is a usual function call
                //  But, if it is OwnFieldAccess (e.g. @something), than this is an OwnMethodCall
                let called_identifier: String;
                let mut is_own_method = false;

                match &boxed_res.expr {
                    Expr::Identifier(ident) => {
                        called_identifier = ident.clone();
                    }
                    Expr::OwnFieldAccess { field } => {
                        called_identifier = field.clone();
                        is_own_method = true;
                    }
                    _ => return perr(self.full_token(0), "Function call of non-function expr"),
                }

                // self.parse_function_call_args checks and consumes both left and right parenthesis
                // As loop condition has already consumed left one, we need to move
                // position back so that self.parse_function_call_args works correctly
                self.position -= 1;
                let args = self.parse_function_call_args()?;

                // Chained function calls (e.g. `object()()` ) are not allowed
                // We need to check those manually, because otherwise next loop iteration will create chained call
                if self.rel_token_check(0, Token::LeftParenthesis) {
                    return perr(
                        self.full_token(0),
                        "No first-class fuctions, chained func calls disallowed",
                    );
                }

                if is_own_method {
                    inner = Expr::OwnMethodCall { method: called_identifier, args };
                } else {
                    inner = Expr::FunctionCall { function: called_identifier, args };
                }
            }

            res_expr = self.expr_with_pos(inner, start, self.position - 1)?;
        }

        Ok(res_expr)
    }

    fn parse_group_or_tuple(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        consume_and_check!(self, Token::LeftParenthesis);
        let mut result_expr = self.parse_expr()?;

        // If comma - then this is not just grouping, but a tuple
        if self.rel_token_check(0, Token::Comma) {
            let mut tuple_exprs: Vec<ExprWithPos> = vec![result_expr];

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
                result_expr = self.expr_with_pos(
                    Expr::TupleValue(tuple_exprs),
                    start,
                    // self.position instead of -1 because we are gonna consume right parenthesis now
                    self.position,
                )?;
            }
        }

        consume_and_check!(self, Token::RightParenthesis);
        Ok(result_expr)
    }

    fn parse_list_literal(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        consume_and_check!(self, Token::LeftSquareBrackets);
        let mut list_items: Vec<ExprWithPos> = vec![];

        until_closes!(self, Token::RightSquareBrackets, {
            list_items.push(self.parse_expr()?);
            consume_if_matches_one_of!(self, [Token::Comma]);
        });

        self.expr_with_pos(Expr::ListValue(list_items), start, self.position - 1)
    }

    fn parse_new_class_instance_expr(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let typename = consume_and_check_type_ident!(self);
        let args = self.parse_function_call_args()?;
        self.expr_with_pos(
            Expr::NewClassInstance { typename, args },
            start,
            self.position - 1,
        )
    }

    fn parse_spawn_active_expr(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        consume_and_check!(self, Token::Spawn);
        let typename = consume_and_check_type_ident!(self);
        let args = self.parse_function_call_args()?;
        self.expr_with_pos(
            Expr::SpawnActive { typename, args },
            start,
            self.position - 1,
        )
    }

    pub fn parse_expr_primary(&mut self) -> ParseResult<ExprWithPos> {
        let start = self.position;
        let expr = match self.rel_token(0) {
            Token::This => Expr::This,
            Token::Float(f) => Expr::Float(*f),
            Token::Integer(i) => Expr::Int(*i),
            Token::String(s) => Expr::String(s.clone()),
            Token::Nil => Expr::Nil,
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
            Token::Identifier(i) => Expr::Identifier(i.clone()),
            Token::OwnIdentifier(f) => Expr::OwnFieldAccess { field: f.clone() },
            Token::LeftParenthesis => return self.parse_group_or_tuple(),
            Token::LeftSquareBrackets => return self.parse_list_literal(),
            Token::TypeIdentifier(_) => return self.parse_new_class_instance_expr(),
            Token::Spawn => return self.parse_spawn_active_expr(),
            _ => {
                return perr(self.full_token(0), "Can't parse expression");
            }
        };

        self.consume_token();

        self.expr_with_pos(expr, start, self.position - 1)
    }
}
