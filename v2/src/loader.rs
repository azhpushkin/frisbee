use crate::ast::*;
use crate::{errors, parser, scanner};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type ModulePath = Vec<String>;
pub fn alias(mp: &ModulePath) -> String {
    mp.join(".")
}

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
    pub main_module: String,
    pub files: HashMap<String, LoadedFile>,
}

fn load_file(workdir: &PathBuf, module_path: ModulePath) -> Option<LoadedFile> {
    println!(" ... Loading {}", alias(&module_path));
    let mut file_path = workdir.to_owned();
    for subpath in &module_path {
        file_path.push(subpath);
    }
    file_path.set_extension("frisbee");

    let contents = std::fs::read_to_string(&file_path).expect("Cant read file");

    let tokens = scanner::scan_tokens(&contents);
    if tokens.is_err() {
        errors::show_scan_error(&contents, &alias(&module_path), tokens.unwrap_err());
        return None;
    }

    let ast: parser::ParseResult<FileAst> = parser::parse(tokens.unwrap());

    if ast.is_err() {
        errors::show_parse_error(&contents, &alias(&module_path), ast.unwrap_err());
        return None;
    }

    Some(LoadedFile { path: file_path, module_path, contents, ast: ast.unwrap() })
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
        main_module: main_module.to_owned(),
        files: HashMap::new(),
    };

    let mut modules_to_load: Vec<ModulePath> = vec![vec![main_module.to_owned()]];

    while !modules_to_load.is_empty() {
        let module_path = modules_to_load.pop().unwrap();

        let loaded_file = load_file(&whole_program.workdir, module_path);
        if loaded_file.is_none() {
            return None;
        }

        let loaded_file = loaded_file.unwrap();
        let loaded_file_alias = alias(&loaded_file.module_path);

        whole_program
            .files
            .insert(loaded_file_alias.clone(), loaded_file);

        let loaded_file = whole_program.files.get(&loaded_file_alias).unwrap();

        for import in &loaded_file.ast.imports {
            // todo swap [0] to correct path forming
            let a = alias(&import.module_path);

            if whole_program.files.get(&a).is_none() {
                modules_to_load.push(import.module_path.clone());
            } else {
                println!("Using cache for {}", a);
            }
        }
    }
    Some(whole_program)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::TestFilesCreator;

    #[test]
    #[should_panic] // TODO: proper error reporting check
    fn import_of_missing_file() {
        let mut files_dir = TestFilesCreator::new();
        files_dir.set_mainfile("from mod import somefun;");

        load_program(files_dir.get_main_path());
    }
}
