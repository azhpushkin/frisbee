use crate::ast::*;
use crate::semantic_checker::semantic_error::SemanticResult;

use super::aggregate::ProgramAggregate;
use super::expressions::ExprTypeChecker;
use super::light_ast::LStatement;
use super::semantic_error::sem_err;
use super::symbols::GlobalSymbolsInfo;

pub fn generate_light_statements(
    original_function: &FunctionDecl,
    file_module: &ModulePathAlias,
    scope: Option<String>,
    aggregate: &ProgramAggregate,
) -> Vec<LStatement> {
    let mut lights: Vec<LStatement> = vec![];

    for statement in original_function {

    }

    lights
}