use std::collections::HashMap;

use crate::loader::{generate_alias, LoadedFile, ModuleAlias, WholeProgram};
use crate::semantics::errors::SemanticError;

use super::errors::{top_level_with_module, SemanticResultWithModule};
use super::std_definitions::is_std_function;
use super::symbols::{SymbolFunc, SymbolType};

type SymbolLookupMapping<T> = HashMap<ModuleAlias, HashMap<String, T>>;

type SingleFileMapping<T> = HashMap<String, T>;

pub type SymbolResolver<'a, T> = Box<dyn Fn(&String) -> T + 'a>;

trait Symbol {}

impl Symbol for SymbolType {}
impl Symbol for SymbolFunc {}

pub struct NameResolver {
    // key is where the symbol lookup occures, value is target
    typenames: SymbolLookupMapping<SymbolType>,
    functions: SymbolLookupMapping<SymbolFunc>,
}

impl NameResolver {
    pub fn create(wp: &WholeProgram) -> SemanticResultWithModule<NameResolver> {
        let mut resolver = NameResolver { typenames: HashMap::new(), functions: HashMap::new() };

        for (file_name, file) in wp.files.iter() {
            check_module_does_not_import_itself(file)?;

            let function_origins = get_functions_origins(file);
            let functions_mapping: SingleFileMapping<SymbolFunc> =
                get_origins(function_origins, &SymbolFunc::new)?;

            let typename_origins = get_typenames_origins(file);
            let typenames_mapping: SingleFileMapping<SymbolType> =
                get_origins(typename_origins, &SymbolType::new)?;

            resolver.functions.insert(file_name.clone(), functions_mapping);
            resolver.typenames.insert(file_name.clone(), typenames_mapping);
        }

        resolver.validate(&wp)?;

        Ok(resolver)
    }

    pub fn get_typenames_resolver<'a, 'b, 'c>(
        &'a self,
        alias: &'b ModuleAlias,
    ) -> SymbolResolver<'c, SymbolType>
    where
        'a: 'c,
        'b: 'c,
    {
        Box::new(move |name: &String| {
            let typename: Option<&SymbolType> = self.typenames[&alias].get(name);
            match typename {
                Some(t) => t.clone(),
                None => panic!("Type {} not found in {}", name, alias),
            }
        })
    }

    pub fn get_functions_resolver<'a, 'b, 'c>(
        &'a self,
        alias: &'b ModuleAlias,
    ) -> SymbolResolver<'c, SymbolFunc>
    where
        'a: 'c,
        'b: 'c,
    {
        Box::new(move |name: &String| {
            let function: Option<&SymbolFunc> = self.functions[&alias].get(name);
            match function {
                Some(f) => f.clone(),
                None => panic!("Function {} not found in {}", name, alias),
            }
        })
    }

    fn validate(&self, wp: &WholeProgram) -> SemanticResultWithModule<()> {
        for (alias, file) in wp.files.iter() {
            for (module, name) in get_functions_origins(file) {
                if !self.functions[&module].contains_key(name) {
                    return top_level_with_module!(
                        alias,
                        "Imported function {} is not defined in module {}!",
                        name,
                        module
                    );
                }

                if is_std_function(name) {
                    return top_level_with_module!(
                        alias,
                        "Function {} is already defined in stdlib!",
                        name
                    );
                }
            }
            for (module, typename) in get_typenames_origins(file) {
                if !self.typenames[&module].contains_key(typename) {
                    return top_level_with_module!(
                        alias,
                        "Imported type {} is not defined in module {}!",
                        typename,
                        module
                    );
                }
            }
        }
        Ok(())
    }
}

fn check_module_does_not_import_itself(file: &LoadedFile) -> SemanticResultWithModule<()> {
    for import in &file.ast.imports {
        if generate_alias(&import.module_path) == file.module_alias {
            return top_level_with_module!(
                file.module_alias,
                "Module {} is importing itself!",
                file.module_alias
            );
        }
    }
    Ok(())
}

fn get_origins<'a, I, T>(
    symbols_origins: I,
    compile_symbol: &dyn Fn(&ModuleAlias, &'a str) -> T,
) -> SemanticResultWithModule<SingleFileMapping<T>>
where
    I: Iterator<Item = (ModuleAlias, &'a str)>,
{
    let mut mapping: HashMap<String, T> = HashMap::new();

    for (module_alias, symbol) in symbols_origins {
        if mapping.contains_key(symbol) {
            return top_level_with_module!(module_alias, "Symbol {} introduces more than once", symbol,);
        }
        mapping.insert(symbol.to_owned(), compile_symbol(&module_alias, symbol));
    }

    Ok(mapping)
}

fn get_typenames_origins<'a>(
    file: &'a LoadedFile,
) -> Box<dyn Iterator<Item = (ModuleAlias, &'a str)> + 'a> {
    let defined_types = file
        .ast
        .types
        .iter()
        .map(move |d| (file.module_alias.clone(), d.name.as_str()));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.typenames
            .iter()
            .map(move |typename| (generate_alias(&i.module_path), typename.as_str()))
    });
    // TODO: we might have some more complex std types as well as functions

    Box::new(defined_types.chain(imported_types))
}

fn get_functions_origins<'a>(
    file: &'a LoadedFile,
) -> Box<dyn Iterator<Item = (ModuleAlias, &'a str)> + 'a> {
    let defined_functions = file
        .ast
        .functions
        .iter()
        .map(move |f| (file.module_alias.clone(), f.name.as_str()));

    let imported_functions = file.ast.imports.iter().flat_map(|i| {
        i.functions
            .iter()
            .map(move |funcname| (generate_alias(&i.module_path), funcname.as_str()))
    });

    Box::new(defined_functions.chain(imported_functions))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{new_alias, setup_and_load_program};

    #[test]
    pub fn check_resolver_mappings() {
        let wp = setup_and_load_program(
            r#"
            ===== file: main.frisbee
            from mod import somefun;
    
            class SomeType {}
            ===== file: mod.frisbee
            fun Nil somefun() {}
        "#,
        );

        let resolver = NameResolver::create(&wp).unwrap();

        let main_alias = new_alias("main");
        let mod_alias = new_alias("mod");

        let main_types_resolver = resolver.get_typenames_resolver(&main_alias);
        assert_eq!(
            main_types_resolver(&String::from("SomeType")),
            SymbolType::new(&main_alias, &String::from("SomeType"))
        );

        let main_functions_resolver = resolver.get_functions_resolver(&main_alias);
        let mod_functions_resolver = resolver.get_functions_resolver(&mod_alias);
        assert_eq!(
            main_functions_resolver(&String::from("somefun")),
            SymbolFunc::new(&mod_alias, &String::from("somefun"))
        );
        assert_eq!(
            mod_functions_resolver(&String::from("somefun")),
            SymbolFunc::new(&mod_alias, &String::from("somefun"))
        );
    }

    #[test]
    pub fn check_validate() {
        let wp = setup_and_load_program(
            r#"
            ===== file: main.frisbee
            from mod import somefun_not_existing;
            ===== file: mod.frisbee
            fun Nil somefun() {}
        "#,
        );
        assert!(NameResolver::create(&wp).is_err());
    }
}
