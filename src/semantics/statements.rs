use crate::ast::*;

use super::aggregate::ProgramAggregate;
use super::light_ast::LStatement;
use super::resolvers::Symbol;


pub fn fill_r_functions(wp: &WholeProgram, aggregate: &mut ProgramAggregate) {
    for (_, func_decl) in aggregate.functions.iter_mut() {
        let l_statements = generate_light_statements(, , scope, aggregate)
    }
}

pub fn generate_light_statements(
    original_function: &FunctionDecl,
    file_module: &ModulePathAlias,
    scope: Option<Symbol>,
    aggregate: &ProgramAggregate,
) -> Vec<LStatement> {
    let mut lights: Vec<LStatement> = vec![];

    for statement in original_function {

    }

    lights
}