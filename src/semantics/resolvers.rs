use std::collections::HashMap;

use crate::ast::{ModulePathAlias, Type};
use crate::loader::{LoadedFile, WholeProgram};

use super::symbols::{SymbolFunc, SymbolType};

type SymbolLookupMapping<T>
where
    T: Symbol,
= HashMap<String, HashMap<String, T>>;

type SingleFileMapping<T>
where
    T: Symbol,
= HashMap<String, T>;

pub type SymbolResolver<'a, T>
where
    T: Symbol,
= Box<dyn Fn(&String) -> T + 'a>;

trait Symbol {}

impl Symbol for SymbolType {}
impl Symbol for SymbolFunc {}

pub struct NameResolver {
    // key is where the symbol lookup occures, value is target
    typenames: SymbolLookupMapping<SymbolType>,
    functions: SymbolLookupMapping<SymbolFunc>,
}

impl NameResolver {
    pub fn create(wp: &WholeProgram) -> NameResolver {
        let mut resolver = NameResolver { typenames: HashMap::new(), functions: HashMap::new() };

        for (file_name, file) in wp.files.iter() {
            check_module_does_not_import_itself(file);

            let function_origins = get_functions_origins(file);
            let functions_mapping = get_origins(function_origins, &SymbolFunc::new)
                .unwrap_or_else(|x| panic!("Function {} defined twice in {}", x, file_name.0));

            let typename_origins = get_typenames_origins(file);
            let typenames_mapping = get_origins(typename_origins, &SymbolType::new)
                .unwrap_or_else(|x| panic!("Type {} defined twice in {}", x, file_name.0));

            resolver.functions.insert(file_name.0.clone(), functions_mapping);
            resolver.typenames.insert(file_name.0.clone(), typenames_mapping);
        }

        resolver.validate(&wp);

        resolver
    }

    pub fn get_typenames_resolver<'a, 'b, 'c>(
        &'a self,
        alias: &'b ModulePathAlias,
    ) -> SymbolResolver<'c, SymbolType>
    where
        'a: 'c,
        'b: 'c,
    {
        Box::new(move |name: &String| {
            let typename = self.typenames[&alias.0].get(name);
            match typename {
                Some(t) => t.clone(),
                None => panic!("Type {} not found in {}", name, alias.0),
            }
        })
    }

    pub fn get_functions_resolver<'a, 'b, 'c>(
        &'a self,
        alias: &'b ModulePathAlias,
    ) -> SymbolResolver<'c, SymbolFunc>
    where
        'a: 'c,
        'b: 'c,
    {
        Box::new(move |name: &String| {
            let function = self.functions[&alias.0].get(name);
            match function {
                Some(f) => f.clone(),
                None => panic!("Function {} not found in {}", name, alias.0),
            }
        })
    }

    fn validate(&self, wp: &WholeProgram) {
        for (_, file) in wp.files.iter() {
            check_module_does_not_import_itself(file);

            for (module, name) in get_functions_origins(file) {
                if !self.functions[&module.0].contains_key(name) {
                    panic!(
                        "Expected function {} to be defined in module {:?}!",
                        name, module
                    );
                }
            }
            for (module, typename) in get_typenames_origins(file) {
                if !self.typenames[&module.0].contains_key(typename) {
                    panic!(
                        "Expected type {} to be defined in module {:?}!",
                        typename, module
                    );
                }
            }
        }
    }
}

fn check_module_does_not_import_itself(file: &LoadedFile) {
    for import in &file.ast.imports {
        if import.module_path == file.module_path {
            panic!("Module {:?} is importing itself!", file.module_path.alias());
        }
    }
}

fn get_origins<'a, I, T>(
    symbols_origins: I,
    compile_symbol: &dyn Fn(&ModulePathAlias, &String) -> T,
) -> Result<SingleFileMapping<T>, String>
where
    I: Iterator<Item = (ModulePathAlias, &'a String)>,
{
    let mut mapping: HashMap<String, T> = HashMap::new();

    for (module_alias, symbol) in symbols_origins {
        if mapping.contains_key(symbol) {
            return Err(symbol.clone());
        }
        mapping.insert(symbol.clone(), compile_symbol(&module_alias, &symbol));
    }

    Ok(mapping)
}

fn get_typenames_origins<'a>(
    file: &'a LoadedFile,
) -> Box<dyn Iterator<Item = (ModulePathAlias, &'a String)> + 'a> {
    let defined_types = file
        .ast
        .types
        .iter()
        .map(move |d| (file.module_path.alias(), &d.name));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.typenames
            .iter()
            .map(move |typename| (i.module_path.alias(), typename))
    });

    Box::new(defined_types.chain(imported_types))
}

fn get_functions_origins<'a>(
    file: &'a LoadedFile,
) -> Box<dyn Iterator<Item = (ModulePathAlias, &'a String)> + 'a> {
    let defined_types = file
        .ast
        .functions
        .iter()
        .map(move |f| (file.module_path.alias(), &f.name));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.functions
            .iter()
            .map(move |funcname| (i.module_path.alias(), funcname))
    });

    Box::new(defined_types.chain(imported_types))
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

        let resolver = NameResolver::create(&wp);

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
    #[should_panic]
    pub fn check_validate() {
        let wp = setup_and_load_program(
            r#"
            ===== file: main.frisbee
            from mod import somefun_not_existing;
            ===== file: mod.frisbee
            fun Nil somefun() {}
        "#,
        );
        NameResolver::create(&wp);
    }
}
