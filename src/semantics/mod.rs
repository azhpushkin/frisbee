use std::collections::HashMap;

use crate::loader::WholeProgram;

mod aggregate;
mod annotations;
mod default_constructors;
mod expressions;
mod light_ast;
mod operators;
mod resolvers;
mod statements;
mod symbols;
mod tests;

pub fn add_default_constructors(wp: &mut WholeProgram) {
    for file in wp.files.values_mut() {
        for class in file.ast.types.iter_mut() {
            default_constructors::add_default_constructor(class);
        }
    }
}

pub fn perform_semantic_analysis(wp: &WholeProgram) -> aggregate::ProgramAggregate {
    let names_resolver = resolvers::NameResolver::create(wp);
    let mut aggregate = aggregate::create_basic_aggregate(wp, &names_resolver);

    let functions_mapping =
        aggregate::fill_aggregate_with_funcs(wp, &mut aggregate, &names_resolver);

    let mut ls_mapping: HashMap<symbols::SymbolFunc, Vec<light_ast::LStatement>> = HashMap::new();

    for (name, raw_function) in aggregate.functions.iter() {
        let light_statements = statements::generate_light_statements(
            &functions_mapping[name].statements,
            raw_function,
            &aggregate,
            &names_resolver,
        );
        ls_mapping.insert(name.clone(), light_statements);
    }

    for (name, raw_function) in aggregate.functions.iter_mut() {
        let light_statements = ls_mapping.remove(name).unwrap();
        raw_function.body = light_statements;
    }

    aggregate
}
