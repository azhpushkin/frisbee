use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::alias::ModuleAlias;
use crate::ast::parsed::*;
use crate::errors::CompileError;
use crate::parsing;

#[derive(Debug)]
pub struct LoadedFile {
    pub path: PathBuf,
    pub module_alias: ModuleAlias,
    pub contents: String,
    pub ast: FileAst,
}

#[derive(Debug)]
pub struct WholeProgram {
    pub workdir: PathBuf,
    pub main_module: ModuleAlias,
    pub files: HashMap<ModuleAlias, LoadedFile>,
}

impl WholeProgram {
    pub fn iter(&self) -> impl Iterator<Item = (&ModuleAlias, &FileAst)> {
        self.files.iter().map(|(k, v)| (k, &v.ast))
    }
}

fn load_file<'a>(
    workdir: &PathBuf,
    module_path: &Vec<String>,
) -> Result<LoadedFile, (ModuleAlias, String, Box<dyn CompileError>)> {
    if module_path.first().unwrap() == "std" {
        // TODO: do something with this?
        panic!("Error loading {:?}: std is reserved", module_path);
    }
    // TODO: implement logging system for this
    let mut file_path = workdir.to_owned();
    for subpath in module_path.iter() {
        file_path.push(subpath);
    }
    file_path.set_extension("frisbee");

    let contents = std::fs::read_to_string(&file_path).expect("Cant read file");
    let module_alias = ModuleAlias::new(module_path);

    let (tokens, scan_status) = parsing::scanner::scan_tokens(&contents);
    if let Err(e) = scan_status {
        return Err((module_alias, contents, Box::new(e)));
    }    

    let ast = parsing::parse_file(&tokens);
    let ast = match ast {
        Ok(ast) => ast,
        Err(e) => return Err((module_alias, contents, Box::new(e))),
    };

    Ok(LoadedFile { path: file_path, module_alias, contents, ast })
}

// TODO:  ensure both windows and Unix are working file
pub fn load_program<'a>(
    entry_file_path: &Path,
) -> Result<WholeProgram, (ModuleAlias, String, Box<dyn CompileError>)> {
    let workdir = entry_file_path.parent().unwrap();

    if entry_file_path.extension().unwrap() != "frisbee" {
        panic!(
            "Only *.frisbee files are allowed, but got {:?}!",
            entry_file_path.extension()
        );
    };

    // TODO: file_stem returns OsString, but I convert it to str
    // need to check how this works under windows/macos
    let main_module = entry_file_path.file_stem().unwrap().to_str().unwrap();

    let mut whole_program = WholeProgram {
        workdir: workdir.to_owned(),
        main_module: ModuleAlias::new(&[main_module.to_owned()]),
        files: HashMap::new(),
    };

    let mut modules_to_load: Vec<Vec<String>> = vec![vec![main_module.to_owned()]];

    while !modules_to_load.is_empty() {
        let module_path = modules_to_load.pop().unwrap();

        // TODO: check error reporting over here
        let loaded_file = load_file(&whole_program.workdir, &module_path)?;

        let alias = ModuleAlias::new(&module_path);

        whole_program.files.insert(alias.clone(), loaded_file);

        let loaded_file = whole_program.files.get(&alias).unwrap();

        for import in &loaded_file.ast.imports {
            // todo swap [0] to correct path forming
            let alias = ModuleAlias::new(&import.module_path);

            if whole_program.files.get(&alias).is_none() {
                modules_to_load.push(import.module_path.clone());
            } else {
                println!("Using cache for {}", alias);
            }
        }
    }
    Ok(whole_program)
}

#[cfg(test)]
mod test {
    use crate::test_utils::{setup_and_load_program, TestFilesCreator};

    use super::*;

    #[test]
    #[should_panic] // TODO: proper error reporting check
    fn import_of_missing_file() {
        let mut files_dir = TestFilesCreator::new();
        files_dir.set_mainfile("from mod import somefun;");

        load_program(files_dir.get_main_path()).unwrap();
    }

    #[test]
    fn check_loading_of_files() {
        let wp = setup_and_load_program(
            r#"
            ===== file: main.frisbee
            from sub.mod import Type;

            class Main {}
            ===== file: sub/mod.frisbee
            active Type {}
        "#,
        );
        assert_eq!(wp.files.len(), 2);

        let main_module_alias = ModuleAlias::new(&["main".into()]);
        let sub_mod_module_alias = ModuleAlias::new(&["sub".into(), "mod".into()]);
        assert_eq!(wp.main_module, main_module_alias);

        let main_file = &wp.files[&main_module_alias];
        let sub_mod_file = &wp.files[&sub_mod_module_alias];

        assert_eq!(main_file.path, wp.workdir.join("main.frisbee"));
        assert_eq!(main_file.module_alias, main_module_alias);

        assert_eq!(
            sub_mod_file.path,
            wp.workdir.join("sub").join("mod.frisbee")
        );
        assert_eq!(sub_mod_file.module_alias, sub_mod_module_alias);
    }
}
