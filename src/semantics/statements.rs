use crate::ast::*;
use crate::loader::WholeProgram;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::light_ast::LStatement;
use super::resolvers::{Symbol, NameResolver};


pub fn generate_light_statements(
    original_function: &FunctionDecl,
    aggregate: &ProgramAggregate,
    resolver: &NameResolver,
) -> Vec<LStatement> {
    // let resolve_type_symbol = resolver.get_typenames_resolver(&new_function.defined_at);
    // let resolve_func_symbol = resolver.get_functions_resolver(&new_function.defined_at);

    let mut lights: Vec<LStatement> = vec![];

    // for statement in original_function {

    // }

    lights
}