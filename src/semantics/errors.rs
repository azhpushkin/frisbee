use crate::alias::ModuleAlias;
use crate::ast::parsed::{ExprWithPos, StatementWithPos};

#[derive(Debug, Clone)]
pub enum SemanticError {
    ExprError { pos_first: usize, pos_last: usize, message: String },
    StmtError { pos: usize, message: String },
    TopLevelError { pos: usize, message: String },
}

#[derive(Debug)]
pub struct SemanticErrorWithModule {
    pub module: ModuleAlias,
    pub error: SemanticError,
}

impl SemanticError {
    pub fn add_statement<'a>(
        stmt: &'a StatementWithPos,
    ) -> Box<dyn Fn(String) -> SemanticError + 'a> {
        Box::new(move |msg: String| -> SemanticError {
            SemanticError::StmtError { pos: stmt.pos, message: msg }
        })
    }

    pub fn add_expr<'a>(expr: &'a ExprWithPos) -> Box<dyn Fn(String) -> SemanticError + 'a> {
        Box::new(move |msg: String| -> SemanticError {
            SemanticError::ExprError {
                pos_first: expr.pos_first,
                pos_last: expr.pos_last,
                message: msg,
            }
        })
    }

    pub fn to_top_level(pos: usize, s: String) -> SemanticError {
        SemanticError::TopLevelError { pos, message: s }
    }

    pub fn with_module(self, module: &ModuleAlias) -> SemanticErrorWithModule {
        SemanticErrorWithModule { module: module.clone(), error: self }
    }
}

pub type SemanticResult<T> = Result<T, SemanticError>;

macro_rules! top_level_with_module {
    ($module:expr, $decl:ident, $($arg:tt)*) => {
        Err(SemanticErrorWithModule {
            module: ($module).clone(),
            error: crate::semantics::errors::SemanticError::TopLevelError {
                message: format!($($arg)*),
                pos: $decl.pos
            }
        })
    };
}
macro_rules! statement_error {
    ($statement:expr, $($arg:tt)*) => {
        Err(crate::semantics::errors::SemanticError::StmtError {
            message: format!($($arg)*),
            pos: ($statement).pos
        })
    };
}
macro_rules! expression_error {
    ($expression:expr, $($arg:tt)*) => {
        Err(crate::semantics::errors::SemanticError::ExprError {
            message: format!($($arg)*),
            pos_first: ($expression).pos_first,
            pos_last: ($expression).pos_last,
        })
    };
}
pub(crate) use expression_error;
pub(crate) use statement_error;
pub(crate) use top_level_with_module;
