use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ast::*;
use crate::{errors, parser, scanner};

#[derive(Debug)]
pub struct LoadedFile {
    pub path: PathBuf,
    pub module_path: ModulePath,
    pub contents: String,
    pub ast: FileAst,
}

#[derive(Debug)]
pub struct WholeProgram {
    pub workdir: PathBuf,
    pub main_module: ModulePath,
    pub files: HashMap<ModulePathAlias, LoadedFile>,
}

fn load_file(workdir: &PathBuf, module_path: &ModulePath) -> Option<LoadedFile> {
    // TODO: implement logging system for this
    println!(" ... Loading {}", module_path.alias_str());
    let mut file_path = workdir.to_owned();
    for subpath in module_path.0.iter() {
        file_path.push(subpath);
    }
    file_path.set_extension("frisbee");

    let contents = std::fs::read_to_string(&file_path).expect("Cant read file");

    let tokens = scanner::scan_tokens(&contents);
    if tokens.is_err() {
        errors::show_scan_error(&contents, &module_path, tokens.unwrap_err());
        return None;
    }

    let ast: parser::ParseResult<FileAst> = parser::parse(tokens.unwrap());

    if ast.is_err() {
        errors::show_parse_error(&contents, &module_path, ast.unwrap_err());
        return None;
    }
    println!(" ... {} loaded!", module_path.alias().0);

    Some(LoadedFile {
        path: file_path,
        module_path: module_path.clone(),
        contents,
        ast: ast.unwrap(),
    })
}

// TODO:  ensure both windows and Unix are working file
pub fn load_program(entry_file_path: &Path) -> Option<WholeProgram> {
    let workdir = entry_file_path.parent().unwrap();

    if entry_file_path.extension().unwrap() != "frisbee" {
        panic!(
            "Only *.frisbee files are allowed, but got {:?}!",
            entry_file_path.extension()
        );
    };

    let main_module = entry_file_path.file_stem().unwrap().to_str().unwrap();
    let mut whole_program = WholeProgram {
        workdir: workdir.to_owned(),
        main_module: ModulePath(vec![main_module.into()]),
        files: HashMap::new(),
    };

    let mut modules_to_load: Vec<ModulePath> = vec![whole_program.main_module.clone()];

    while !modules_to_load.is_empty() {
        let module_path = modules_to_load.pop().unwrap();

        let loaded_file = load_file(&whole_program.workdir, &module_path);
        if loaded_file.is_none() {
            return None;
        }

        let loaded_file = loaded_file.unwrap();

        whole_program
            .files
            .insert(module_path.alias().clone(), loaded_file);

        let loaded_file = whole_program.files.get(&module_path.alias()).unwrap();

        for import in &loaded_file.ast.imports {
            // todo swap [0] to correct path forming
            let alias = import.module_path.alias();

            if whole_program.files.get(&alias).is_none() {
                modules_to_load.push(import.module_path.clone());
            } else {
                println!("Using cache for {}", alias.0);
            }
        }
    }
    Some(whole_program)
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

        load_program(files_dir.get_main_path());
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

        let main_module_path = ModulePath(vec!["main".into()]);
        let sub_mod_module_path = ModulePath(vec!["sub".into(), "mod".into()]);
        assert_eq!(wp.main_module, main_module_path);

        let main_file = &wp.files[&main_module_path.alias()];
        let sub_mod_file = &wp.files[&sub_mod_module_path.alias()];

        assert_eq!(main_file.path, wp.workdir.join("main.frisbee"));
        assert_eq!(main_file.module_path, main_module_path);

        assert_eq!(
            sub_mod_file.path,
            wp.workdir.join("sub").join("mod.frisbee")
        );
        assert_eq!(sub_mod_file.module_path, sub_mod_module_path);
    }
}
