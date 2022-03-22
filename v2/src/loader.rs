use crate::ast::*;
use crate::{errors, parser, scanner};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LoadedFile {
    pub path: PathBuf,
    pub module_name: String,
    pub contents: String,
    pub ast: FileAst,
}

#[derive(Debug)]
pub struct WholeProgram {
    pub workdir: PathBuf,
    pub mainfile: String,
    pub files: HashMap<String, LoadedFile>,
}

pub fn load_file(path: &PathBuf) -> Option<LoadedFile> {
    println!(" ... Loading {}", path.to_str().unwrap());

    let contents = std::fs::read_to_string(path).expect("Cant read file");
    let module_name = path_to_module(&path);

    let tokens = scanner::scan_tokens(&contents);
    if tokens.is_err() {
        errors::show_scan_error(&contents, &module_name, tokens.unwrap_err());
        return None;
    }

    let ast: parser::ParseResult<FileAst> = parser::parse(tokens.unwrap());

    if ast.is_err() {
        errors::show_parse_error(&contents, &module_name, ast.unwrap_err());
        return None;
    }

    Some(LoadedFile { path: path.clone(), module_name, contents, ast: ast.unwrap() })
}

pub fn path_to_module(path: &PathBuf) -> String {
    return path.file_name().unwrap().to_str().unwrap().into();
}

// TODO:  ensure both windows and Unix are working file
pub fn load_program(entry_file_path: &Path) {
    let workdir = entry_file_path.parent().unwrap();
    let filename = entry_file_path.file_name().unwrap().to_str().unwrap();
    let mut whole_program = WholeProgram {
        workdir: workdir.to_owned(),
        mainfile: String::from(filename),
        files: HashMap::new(),
    };

    let mut modules_to_load: Vec<PathBuf> = vec![entry_file_path.to_owned()];

    while !modules_to_load.is_empty() {
        let path_to_load = modules_to_load.pop().unwrap();
        let loaded_file = load_file(&path_to_load);
        if loaded_file.is_none() {
            return;
        }

        let loaded_file = loaded_file.unwrap();

        for import in &loaded_file.ast.imports {
            // todo swap [0] to correct path forming
            let filename = format!("{}.frisbee", import.module_path[0]);
            if whole_program.files.get(&filename).is_none() {
                let mut module_path = whole_program.workdir.clone();
                module_path.push(filename);
                modules_to_load.push(module_path);
            }
        }

        whole_program
            .files
            .insert(path_to_module(&path_to_load), loaded_file);
    }
}
