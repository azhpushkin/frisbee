use crate::ast::{ExprWithPos, StatementWithPos};
use crate::loader::ModuleAlias;

#[derive(Debug)]
pub enum SemanticError {
    ExprError { expr: ExprWithPos, message: String },
    StmtError { stmt: StatementWithPos, message: String },
    TopLevelError { message: String, module: ModuleAlias },
}

pub type SemanticResult<T> = Result<T, SemanticError>;

macro_rules! top_level_error {
    ($module:expr, $($arg:tt)*) => {
        Err(SemanticError::TopLevelError { message: format!($($arg)*), module: ($module).clone() })
    };
}
pub(crate) use top_level_error; // <-- the trick
