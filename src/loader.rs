use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::alias::ModuleAlias;
use crate::ast::parsed::*;
use crate::errors::CompileError;
use crate::parsing;

pub trait FrisbeeModuleLoader {
    fn load_module(&self, module: &ModuleAlias) -> Result<String, String>;
}

pub struct FileSystemLoader {
    pub workdir: PathBuf,
}

impl FrisbeeModuleLoader for FileSystemLoader {
    fn load_module(&self, module: &ModuleAlias) -> Result<String, String> {
        let mut file_path = self.workdir.to_owned();
        for subpath in module.to_path().iter() {
            file_path.push(subpath);
        }
        file_path.set_extension("frisbee");

        std::fs::read_to_string(&file_path).map_err(|err| format!("{}", err))
    }
}

#[derive(Debug)]
pub struct FrisbeeModule {
    pub alias: ModuleAlias,
    pub contents: String,
    pub ast: FileAst,
}

#[derive(Debug)]
pub struct WholeProgram {
    pub main_module: ModuleAlias,
    pub modules: HashMap<ModuleAlias, FrisbeeModule>,
}

impl WholeProgram {
    pub fn iter(&self) -> impl Iterator<Item = (&ModuleAlias, &FileAst)> {
        self.modules.iter().map(|(k, v)| (k, &v.ast))
    }
}

fn parse_contents(contents: String) -> Result<FileAst, Box<dyn CompileError>> {
    let (tokens, scan_status) = parsing::scanner::scan_tokens(&contents);
    if let Err(e) = scan_status {
        return Err(Box::new(e));
    }

    let ast = parsing::parse_file(&tokens);
    match ast {
        Ok(ast) => Ok(ast),
        Err(e) => return Err(Box::new(e)),
    }
}

pub fn load_modules_recursively(
    loader: &dyn FrisbeeModuleLoader,
    main_module: &ModuleAlias,
) -> Result<WholeProgram, (ModuleAlias, String, Box<dyn CompileError>)> {
    let loaded_modules: HashMap<ModuleAlias, FrisbeeModule> = HashMap::new();

    let mut modules_to_load: Vec<ModuleAlias> = vec![main_module.clone()];

    while let Some(new_module) = modules_to_load.pop() {
        if loaded_modules.contains_key(&new_module) {
            continue;
        }

        let contents = loader.load_module(&new_module).expect("Cannot load module");
        let ast = parse_contents(contents).map_err(|err| (new_module, contents, err))?;
        for import_statement in ast.imports.iter() {
            modules_to_load.push(ModuleAlias::new(&import_statement.module_path))
        }

        loaded_modules.insert(
            new_module,
            FrisbeeModule { alias: new_module, contents, ast },
        );
    }
    Ok(WholeProgram{ main_module: main_module.clone(), modules: loaded_modules })
}

pub fn check_and_aggregate(
    wp: &mut WholeProgram,
) -> Result<
    crate::semantics::aggregate::ProgramAggregate,
    crate::semantics::errors::SemanticErrorWithModule,
> {
    crate::semantics::add_default_constructors(
        wp.modules
            .iter_mut()
            .flat_map(|(_, loaded_file)| loaded_file.ast.types.iter_mut()),
    );
    let modules: Vec<_> = wp.iter().collect();
    crate::semantics::perform_semantic_analysis(&modules, &wp.main_module)
}

// #[cfg(test)]
// mod test {
//     use crate::tests::helpers::{setup_and_load_program, TestFilesCreator};

//     use super::*;

//     #[test]
//     #[should_panic] // TODO: proper error reporting check
//     fn import_of_missing_file() {
//         let mut files_dir = TestFilesCreator::new();
//         files_dir.set_mainfile("from mod import somefun;");

//         load_program(files_dir.get_main_path()).unwrap();
//     }

//     #[test]
//     fn check_loading_of_files() {
//         let wp = setup_and_load_program(
//             r#"
//             ===== file: main.frisbee
//             from sub.mod import Type;

//             class Main {}
//             ===== file: sub/mod.frisbee
//             active Type {}
//         "#,
//         );
//         assert_eq!(wp.modules.len(), 2);

//         let main_module_alias = ModuleAlias::new(&["main".into()]);
//         let sub_mod_module_alias = ModuleAlias::new(&["sub".into(), "mod".into()]);
//         assert_eq!(wp.main_module, main_module_alias);

//         let main_file = &wp.modules[&main_module_alias];
//         let sub_mod_file = &wp.modules[&sub_mod_module_alias];

//         assert_eq!(main_file.path, wp.workdir.join("main.frisbee"));
//         assert_eq!(main_file.module_alias, main_module_alias);

//         assert_eq!(
//             sub_mod_file.path,
//             wp.workdir.join("sub").join("mod.frisbee")
//         );
//         assert_eq!(sub_mod_file.module_alias, sub_mod_module_alias);
//     }
// }
