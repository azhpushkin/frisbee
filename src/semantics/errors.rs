use crate::ast::{ExprWithPos, StatementWithPos};
use crate::loader::ModuleAlias;

#[derive(Debug)]
pub enum SemanticError {
    ExprError { expr: ExprWithPos, message: String },
    StmtError { stmt: StatementWithPos, message: String },
    TopLevelError { message: String },
}

pub type SemanticResult<T> = Result<T, SemanticError>;
pub type SemanticResultWithModule<T> = Result<T, (ModuleAlias, SemanticError)>;

macro_rules! top_level_with_module {
    ($module:expr, $($arg:tt)*) => {
        Err((($module).clone(), SemanticError::TopLevelError { message: format!($($arg)*) }))
    };
}
macro_rules! top_level_error {
    ($($arg:tt)*) => {
        Err(SemanticError::TopLevelError { message: format!($($arg)*)})
    };
}
macro_rules! statement_error {
    ($statement:expr, $($arg:tt)*) => {
        Err(SemanticError::Stmt { message: format!($($arg)*), stmt: $statement })
    };
}
macro_rules! expresion_error {
    ($expression:expr, $($arg:tt)*) => {
        Err(SemanticError::ExprError { message: format!($($arg)*), expr: $expression })
    };
}
pub(crate) use expresion_error;
pub(crate) use statement_error;
pub(crate) use top_level_error;
pub(crate) use top_level_with_module;
