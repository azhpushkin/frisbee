use crate::loader::WholeProgram;

mod aggregate;
mod default_constructors;
mod light_ast;
mod real_type;
mod resolvers;
mod statements;
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
        

    aggregate
}
