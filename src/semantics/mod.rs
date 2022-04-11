use crate::loader::WholeProgram;

mod aggregate;
mod real_ast;
mod real_type;
mod resolvers;
mod tests;

pub fn perform_semantic_analysis(wp: &WholeProgram) -> aggregate::ProgramAggregate {
    let names_resolver = resolvers::NameResolver::create(wp);
    let aggregate = aggregate::create_basic_aggregate(wp, &names_resolver);

    aggregate
}
