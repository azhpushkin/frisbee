use crate::ast::{StatementWithPos, ExprWithPos};

#[derive(Debug)]
pub enum SemanticError {
    ExprError{expr: ExprWithPos, message: String},
    StmtError{stmt: StatementWithPos, message: String},
    TopLevelError{message: String},
}

impl SemanticError {
    pub fn top_level<T>(message: String) -> SemanticResult<T> {
        Err(SemanticError::TopLevelError{message})
    }
}

pub type SemanticResult<T> = Result<T, SemanticError>;