use crate::ast::*;
use crate::loader::WholeProgram;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::light_ast::LStatement;
use super::resolvers::Symbol;


pub fn get_og_function<'a>(wp: &'a WholeProgram, f: RawFunction) -> &'a FunctionDecl{
    let file = &wp.files[&f.defined_at];
    match f.method_of {
        Some(typename) =>
    }
    
}

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