use crate::alias::ModuleAlias;
use crate::ast::parsed::{ExprWithPos, StatementWithPos};

#[derive(Debug)]
pub enum SemanticError {
    ExprError { pos_first: usize, pos_last: usize, message: String },
    StmtError { pos: usize, message: String },
    TopLevelError { message: String },
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

    pub fn to_top_level(s: String) -> SemanticError {
        SemanticError::TopLevelError { message: s }
    }
}

pub type SemanticResult<T> = Result<T, SemanticError>;
pub type SemanticErrorWithModule = (ModuleAlias, SemanticError);

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
