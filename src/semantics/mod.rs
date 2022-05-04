use std::collections::HashMap;

use crate::alias::ModuleAlias;
use crate::ast;

pub mod aggregate;
mod default_constructors;
pub mod errors;
mod expressions;
mod insights;
mod locals;
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

    let function_names: Vec<_> = aggregate
        .functions
        .iter()
        .map(|(s, f)| (s.clone(), f.defined_at.clone()))
        .collect();
    for (name, module) in function_names {
        statements::verify_raw_function(
            &functions_mapping[&name],
            &name,
            &mut aggregate,
            &names_resolver,
        )
        .map_err(|err| err.with_module(&module))?;
    }

    Ok(aggregate)
}
