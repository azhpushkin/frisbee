use std::collections::HashMap;

use crate::alias::ModuleAlias;
use crate::ast;

pub mod aggregate;
mod default_constructors;
pub mod errors;
mod expressions;
mod insights;
mod operators;
mod resolvers;
mod statements;
mod std_definitions;
mod tests;

pub fn add_default_constructors<'a, I>(classes: I)
where
    I: Iterator<Item = &'a mut ast::parsed::ClassDecl>,
{
    for class in classes {
        default_constructors::add_default_constructor(class);
    }
}

pub fn perform_semantic_analysis(
    modules: &[(&ModuleAlias, &ast::parsed::FileAst)],
    entry_module: &ModuleAlias,
) -> Result<aggregate::ProgramAggregate, errors::SemanticErrorWithModule> {
    let names_resolver = resolvers::NameResolver::create(modules)?;
    let mut aggregate = aggregate::create_basic_aggregate(modules, entry_module, &names_resolver)?;

    let functions_mapping =
        aggregate::fill_aggregate_with_funcs(modules, &mut aggregate, &names_resolver)?;

    let mut ls_mapping: HashMap<crate::symbols::SymbolFunc, Vec<ast::verified::VStatement>> =
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
