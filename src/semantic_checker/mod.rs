use crate::loader::WholeProgram;

mod real_type;
mod real_ast;
mod aggregate;
mod resolvers;



pub fn perform_semantic_analysis(wp: &WholeProgram) -> aggregate::ProgramAggregate {
    let names_resolver = resolvers::NameResolver::create(wp);
    let aggregate = aggregate::create_basic_aggregate(wp, &names_resolver);

    aggregate
}