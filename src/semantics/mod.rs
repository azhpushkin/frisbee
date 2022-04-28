use std::collections::HashMap;

use crate::loader::WholeProgram;

pub mod aggregate;
pub mod annotations;
mod default_constructors;
pub mod errors;
mod expressions;
mod operators;
mod resolvers;
mod statements;
mod std_definitions;
mod tests;
pub mod verified_ast;

pub fn add_default_constructors(wp: &mut WholeProgram) {
    for file in wp.files.values_mut() {
        for class in file.ast.types.iter_mut() {
            default_constructors::add_default_constructor(class);
        }
    }
}

pub fn perform_semantic_analysis(
    wp: &WholeProgram,
) -> Result<aggregate::ProgramAggregate, errors::SemanticErrorWithModule> {
    let names_resolver = resolvers::NameResolver::create(wp)?;
    let mut aggregate = aggregate::create_basic_aggregate(wp, &names_resolver)?;

    let functions_mapping =
        aggregate::fill_aggregate_with_funcs(wp, &mut aggregate, &names_resolver)?;

    let mut ls_mapping: HashMap<crate::symbols::SymbolFunc, Vec<verified_ast::VStatement>> =
        HashMap::new();

    for (name, raw_function) in aggregate.functions.iter() {
        let verified_statements = statements::verify_statements(
            &functions_mapping[name].statements,
            raw_function,
            &aggregate,
            &names_resolver,
        )
        .map_err(|err| (raw_function.defined_at.clone(), err))?;

        ls_mapping.insert(name.clone(), verified_statements);
    }

    for (name, raw_function) in aggregate.functions.iter_mut() {
        let verified_statements = ls_mapping.remove(name).unwrap();
        raw_function.body = verified_statements;
    }

    Ok(aggregate)
}
