use std::collections::HashMap;

use crate::alias::ModuleAlias;
use crate::ast::parsed::FileAst;
use crate::symbols::{SymbolFunc, SymbolType};

use super::errors::{top_level_with_module, SemanticError, SemanticErrorWithModule};

type SymbolLookupMapping<T> = HashMap<ModuleAlias, HashMap<String, T>>;

type SingleFileMapping<T> = HashMap<String, T>;

pub type SymbolResolver<'a, T> = Box<dyn Fn(&str) -> Result<T, String> + 'a>;

trait Symbol {}

impl Symbol for SymbolType {}
impl Symbol for SymbolFunc {}

pub struct NameResolver {
    // key is where the symbol lookup occures, value is target
    typenames: SymbolLookupMapping<SymbolType>,
    functions: SymbolLookupMapping<SymbolFunc>,
}

impl NameResolver {
    pub fn create(
        modules: &[(&ModuleAlias, &FileAst)],
    ) -> Result<NameResolver, SemanticErrorWithModule> {
        let mut resolver = NameResolver { typenames: HashMap::new(), functions: HashMap::new() };

        for (alias, file_ast) in modules.iter() {
            resolver
                .init_file_symbols(*alias, *file_ast)
                .map_err(|err| err.with_module(*alias))?;
        }

        resolver.validate(modules)?;

        Ok(resolver)
    }

    fn init_file_symbols(
        &mut self,
        alias: &ModuleAlias,
        file_ast: &FileAst,
    ) -> Result<(), SemanticError> {
        check_module_does_not_import_itself(alias, file_ast)?;

        let function_origins = get_functions_origins(alias, file_ast);
        let functions_mapping: SingleFileMapping<SymbolFunc> =
            get_origins(function_origins, &SymbolFunc::new).map_err(|(f, pos)| {
                SemanticError::TopLevelError {
                    pos,
                    message: format!("Function {} is already introduced in this module", f),
                }
            })?;

        let typename_origins = get_typenames_origins(alias, file_ast);
        let typenames_mapping: SingleFileMapping<SymbolType> =
            get_origins(typename_origins, &SymbolType::new).map_err(|(f, pos)| {
                SemanticError::TopLevelError {
                    pos,
                    message: format!("Type {} is already introduced in this module", f),
                }
            })?;

        self.functions.insert((*alias).to_owned(), functions_mapping);
        self.typenames.insert((*alias).to_owned(), typenames_mapping);
        Ok(())
    }

    pub fn get_typenames_resolver<'a, 'b, 'c>(
        &'a self,
        alias: &'b ModuleAlias,
    ) -> SymbolResolver<'c, SymbolType>
    where
        'a: 'c,
        'b: 'c,
    {
        Box::new(move |name: &str| {
            let typename: Option<&SymbolType> = self.typenames[alias].get(name);
            match typename {
                Some(t) => Ok(t.clone()),
                None => Err(format!("Type {} not found in {}", name, alias)),
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
        Box::new(move |name: &str| {
            let function: Option<&SymbolFunc> = self.functions[alias].get(name);
            match function {
                Some(f) => Ok(f.clone()),
                None => Err(format!("Function {} not found in {}", name, alias)),
            }
        })
    }

    fn validate(
        &self,
        modules: &[(&ModuleAlias, &FileAst)],
    ) -> Result<(), SemanticErrorWithModule> {
        for (alias, file_ast) in modules.iter() {
            for import_decl in file_ast.imports.iter() {
                let imported_module = ModuleAlias::new(&import_decl.module_path);

                for function_name in import_decl.functions.iter() {
                    if !self.functions[&imported_module].contains_key(function_name) {
                        return top_level_with_module!(
                            *alias,
                            import_decl,
                            "Imported function {} is not defined in module {}!",
                            function_name,
                            imported_module
                        );
                    }
                }

                for typename in import_decl.typenames.iter() {
                    if !self.typenames[&imported_module].contains_key(typename) {
                        return top_level_with_module!(
                            *alias,
                            import_decl,
                            "Imported type {} is not defined in module {}!",
                            typename,
                            imported_module
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

fn check_module_does_not_import_itself(
    alias: &ModuleAlias,
    file_ast: &FileAst,
) -> Result<(), SemanticError> {
    for import in &file_ast.imports {
        if &ModuleAlias::new(&import.module_path) == alias {
            return Err(SemanticError::TopLevelError {
                pos: import.pos,
                message: format!("Module {} is importing itself!", alias),
            });
        }
    }
    Ok(())
}

fn get_origins<'a, I, T>(
    symbols_origins: I,
    compile_symbol: &dyn Fn(&ModuleAlias, &'a str) -> T,
) -> Result<SingleFileMapping<T>, (&'a str, usize)>
where
    I: Iterator<Item = (ModuleAlias, &'a str, usize)>,
{
    let mut mapping: HashMap<String, T> = HashMap::new();

    for (module_alias, symbol, pos) in symbols_origins {
        if mapping.contains_key(symbol) {
            return Err((symbol, pos));
        }
        mapping.insert(symbol.to_owned(), compile_symbol(&module_alias, symbol));
    }

    Ok(mapping)
}

fn get_typenames_origins<'a>(
    alias: &'a ModuleAlias,
    file_ast: &'a FileAst,
) -> Box<dyn Iterator<Item = (ModuleAlias, &'a str, usize)> + 'a> {
    let defined_types = file_ast
        .types
        .iter()
        .map(move |d| (alias.clone(), d.name.as_str(), d.pos));

    let imported_types = file_ast.imports.iter().flat_map(|i| {
        i.typenames
            .iter()
            .map(move |typename| (ModuleAlias::new(&i.module_path), typename.as_str(), i.pos))
    });
    // TODO: we might have some more complex std types as well as functions

    Box::new(defined_types.chain(imported_types))
}

fn get_functions_origins<'a>(
    alias: &'a ModuleAlias,
    file_ast: &'a FileAst,
) -> Box<dyn Iterator<Item = (ModuleAlias, &'a str, usize)> + 'a> {
    let defined_functions = file_ast
        .functions
        .iter()
        .map(move |f| (alias.clone(), f.name.as_str(), f.pos));

    let imported_functions = file_ast.imports.iter().flat_map(|i| {
        i.functions
            .iter()
            .map(move |funcname| (ModuleAlias::new(&i.module_path), funcname.as_str(), i.pos))
    });

    Box::new(defined_functions.chain(imported_functions))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::helpers::{new_alias, setup_and_load_program};

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
        let modules_with_ast: Vec<_> = wp.iter().collect();
        let resolver = NameResolver::create(&modules_with_ast).unwrap();

        let main_alias = new_alias("main");
        let mod_alias = new_alias("mod");

        let main_types_resolver = resolver.get_typenames_resolver(&main_alias);
        assert_eq!(
            main_types_resolver(&String::from("SomeType")).unwrap(),
            SymbolType::new(&main_alias, &String::from("SomeType"))
        );

        let main_functions_resolver = resolver.get_functions_resolver(&main_alias);
        let mod_functions_resolver = resolver.get_functions_resolver(&mod_alias);
        assert_eq!(
            main_functions_resolver(&String::from("somefun")).unwrap(),
            SymbolFunc::new(&mod_alias, &String::from("somefun"))
        );
        assert_eq!(
            mod_functions_resolver(&String::from("somefun")).unwrap(),
            SymbolFunc::new(&mod_alias, &String::from("somefun"))
        );

        assert!(mod_functions_resolver(&String::from("wrong_name")).is_err());
        assert!(main_types_resolver(&String::from("BadType")).is_err());
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
        let modules_with_ast: Vec<_> = wp.iter().collect();
        assert!(NameResolver::create(&modules_with_ast).is_err());
    }
}
