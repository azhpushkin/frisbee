use std::collections::HashMap;

use crate::ast::ModulePathAlias;
use crate::loader::{LoadedFile, WholeProgram};

type SymbolLookupMapping = HashMap<ModulePathAlias, HashMap<String, String>>;
type SingleFileMapping = Result<HashMap<String, String>, String>;

pub type SymbolResolver<'a> = dyn (Fn(&String) -> &'a String) + 'a;

pub struct NameResolver {
    // key is where the symbol lookup occures, value is target
    typenames: SymbolLookupMapping,
    functions: SymbolLookupMapping,
}

pub fn compile_name(alias: &ModulePathAlias, name: &String) -> String {
    format!("{}::{}", alias.0, name)
}
pub fn compile_method(alias: &ModulePathAlias, typename: &String, method: &String) -> String {
    format!("{}::{}::{}", alias.0, typename, method)
}

impl NameResolver {
    pub fn create(wp: &WholeProgram) -> NameResolver {
        let mut resolver = NameResolver { typenames: HashMap::new(), functions: HashMap::new() };

        for (file_name, file) in wp.files.iter() {
            check_module_does_not_import_itself(file);

            let file_functions_mapping = get_functions_origins(file)
                .unwrap_or_else(|x| panic!("Function {} defined twice in {}", x, file_name.0));
            let file_typenames_mapping = get_typenames_origins(file)
                .unwrap_or_else(|x| panic!("Type {} defined twice in {}", x, file_name.0));

            resolver.functions.insert(file_name.clone(), file_functions_mapping);
            resolver.typenames.insert(file_name.clone(), file_typenames_mapping);
        }

        resolver
    }

    pub fn get_typenames_resolver<'a>(
        &'a self,
        alias: &'a ModulePathAlias,
    ) -> Box<SymbolResolver<'a>> {
        Box::new(move |name: &String| self.typenames[alias].get(name).unwrap())
    }

    pub fn get_functions_resolver<'a>(
        &'a self,
        alias: &'a ModulePathAlias,
    ) -> Box<SymbolResolver<'a>> {
        Box::new(move |name: &String| self.functions[alias].get(name).unwrap())
    }
}

fn check_module_does_not_import_itself(file: &LoadedFile) {
    for import in &file.ast.imports {
        if import.module_path == file.module_path {
            panic!("Module {:?} is importing itself!", file.module_path.alias());
        }
    }
}

fn get_origins<'a, I>(symbols_origins: I) -> SingleFileMapping
where
    I: Iterator<Item = (ModulePathAlias, &'a String)>,
{
    let mut mapping: HashMap<String, String> = HashMap::new();

    for (module_alias, symbol) in symbols_origins {
        if mapping.contains_key(symbol) {
            return Err(symbol.clone());
        }
        mapping.insert(symbol.clone(), compile_name(&module_alias, &symbol));
    }

    Ok(mapping)
}

fn get_typenames_origins(file: &LoadedFile) -> SingleFileMapping {
    let defined_types = file.ast.types.iter().map(|d| (file.module_path.alias(), &d.name));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.typenames
            .iter()
            .map(move |typename| (i.module_path.alias(), typename))
    });

    get_origins(defined_types.chain(imported_types))
}

fn get_functions_origins(file: &LoadedFile) -> SingleFileMapping {
    let defined_types = file.ast.functions.iter().map(|f| (file.module_path.alias(), &f.name));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.functions
            .iter()
            .map(move |funcname| (i.module_path.alias(), funcname))
    });

    get_origins(defined_types.chain(imported_types))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::{new_alias, setup_and_load_program};

    pub fn check_resolver_mappings() {
        let wp = setup_and_load_program(
            r#"
            ===== file: main.frisbee
            from mod import somefun;
    
            class SomeType()
            ===== file: mod.frisbee
            fun Nil somefun() {}
        "#,
        );

        let resolver = NameResolver::create(&wp);

        let main_alias = new_alias("main");
        let mod_alias = new_alias("main");

        let main_types_resolver = resolver.get_typenames_resolver(&main_alias);
        assert_eq!(
            main_types_resolver(&String::from("SomeType")),
            "main::SomeType"
        );

        let main_functions_resolver = resolver.get_functions_resolver(&main_alias);
        let mod_functions_resolver = resolver.get_functions_resolver(&mod_alias);
        assert_eq!(
            main_functions_resolver(&String::from("somefun")),
            "mod::somefun"
        );
        assert_eq!(
            mod_functions_resolver(&String::from("somefun")),
            "mod::somefun"
        );
    }
}
