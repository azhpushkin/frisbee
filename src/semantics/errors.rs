use crate::ast::{ExprWithPos, StatementWithPos};
use crate::loader::ModuleAlias;

#[derive(Debug)]
pub enum SemanticError {
    ExprError { pos_first: usize, pos_last: usize, message: String },
    StmtError { pos: usize, message: String },
    TopLevelError { message: String },
}

impl SemanticError {
    pub fn add_statement<'a, T>(
        stmt: &'a StatementWithPos,
    ) -> Box<dyn Fn(String) -> SemanticResult<T> + 'a> {
        Box::new(move |msg: String| -> SemanticResult<T> {
            Err(SemanticError::StmtError { pos: stmt.pos, message: msg })
        })
    }

    pub fn add_expr<'a, T>(expr: &'a ExprWithPos) -> Box<dyn Fn(String) -> SemanticResult<T> + 'a> {
        Box::new(move |msg: String| -> SemanticResult<T> {
            Err(SemanticError::ExprError {
                pos_first: expr.pos_first,
                pos_last: expr.pos_last,
                message: msg,
            })
        })
    }

    pub fn to_top_level<T>(s: String) -> SemanticResult<T> {
        Err(SemanticError::TopLevelError { message: s })
    }
}

pub type SemanticResult<T> = Result<T, SemanticError>;
pub type SemanticResultWithModule<T> = Result<T, (ModuleAlias, SemanticError)>;

macro_rules! top_level_with_module {
    ($module:expr, $($arg:tt)*) => {
        Err((($module).clone(), crate::semantics::errors::SemanticError::TopLevelError { message: format!($($arg)*) }))
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
