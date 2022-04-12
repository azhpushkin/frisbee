use std::collections::HashMap;

use crate::ast::{ModulePathAlias, Type};
use crate::loader::{LoadedFile, WholeProgram};

use super::symbols::{SymbolType, SymbolFunc};

type SymbolOrigin<'a, 'b> = (&'a ModulePathAlias, &'b String);
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

            let file_functions_mapping = get_functions_origins(file)
                .unwrap_or_else(|x| panic!("Function {} defined twice in {}", x, file_name.0));
            let file_typenames_mapping = get_typenames_origins(file)
                .unwrap_or_else(|x| panic!("Type {} defined twice in {}", x, file_name.0));

            resolver.functions.insert(file_name.0.clone(), file_functions_mapping);
            resolver.typenames.insert(file_name.0.clone(), file_typenames_mapping);
        }

        resolver.validate();

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
            println!("== {} {}", &alias.0, &name);
            self.typenames[&alias.0].get(name).unwrap().clone()
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
        Box::new(move |name: &String| self.functions[&alias.0].get(name).unwrap().clone())
    }

    fn validate(&self) {
        let all_typenames = self.typenames.iter().flat_map(|(_, v)| v.values());
        let all_functions = self.functions.iter().flat_map(|(_, v)| v.values());

        for typename in all_typenames {
            let (module, name) = typename.0.split_once("::").unwrap();
            if !self.typenames[module].contains_key(name) {
                panic!("Expected type {} to be defined in module {}!", name, module);
            }
        }

        for function in all_functions {
            let (module, name) = function.0.split_once("::").unwrap();
            if !self.functions[module].contains_key(name) {
                panic!("Expected type {} to be defined in module {}!", name, module);
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
) -> Result<(SingleFileMapping<T>, Vec<I>), String>
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

fn get_typenames_origins(file: &LoadedFile) -> Result<SingleFileMapping<SymbolType>, String> {
    let defined_types = file.ast.types.iter().map(|d| (file.module_path.alias(), &d.name));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.typenames
            .iter()
            .map(move |typename| (i.module_path.alias(), typename))
    });

    get_origins(defined_types.chain(imported_types), &compile_typename)
}

fn get_functions_origins(file: &LoadedFile) -> Result<SingleFileMapping<SymbolFunc>, String> {
    let defined_types = file.ast.functions.iter().map(|f| (file.module_path.alias(), &f.name));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.functions
            .iter()
            .map(move |funcname| (i.module_path.alias(), funcname))
    });

    get_origins(defined_types.chain(imported_types), &compile_func)
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
            SymbolType("main::SomeType".into())
        );

        let main_functions_resolver = resolver.get_functions_resolver(&main_alias);
        let mod_functions_resolver = resolver.get_functions_resolver(&mod_alias);
        assert_eq!(
            main_functions_resolver(&String::from("somefun")),
            SymbolFunc("mod::somefun".into())
        );
        assert_eq!(
            mod_functions_resolver(&String::from("somefun")),
            SymbolFunc("mod::somefun".into())
        );
    }
}
