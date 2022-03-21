use crate::ast::*;
use crate::{errors, parser, scanner};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct LoadedFile {
    pub path: PathBuf,
    pub module_name: String,
    pub contents: String,
    pub ast: Option<Program>,
}

impl LoadedFile {
    pub fn new_unloaded(path: PathBuf) -> LoadedFile {
        LoadedFile {
            path: path.clone(),
            module_name: path.file_name().unwrap().to_str().unwrap().into(),
            contents: String::from(""),
            ast: None,
        }
    }
}

pub struct WholeProgram {
    pub workdir: PathBuf,
    pub mainfile: String,
    pub files: HashMap<String, LoadedFile>,
}

pub fn generate_ast(file: &mut LoadedFile) {
    println!(" ... Loading {}", file.path.to_str().unwrap());

    file.contents = std::fs::read_to_string(&file.path).expect("Cant read file");

    let tokens = scanner::scan_tokens(&file.contents);
    if tokens.is_err() {
        errors::show_scan_error(&file, tokens.unwrap_err());
        return;
    }

    let ast: parser::ParseResult<Program> = parser::parse(tokens.unwrap());

    if ast.is_err() {
        errors::show_parse_error(&file, ast.unwrap_err());
        return;
    }

    file.ast = Some(ast.unwrap());
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

    let x = LoadedFile::new_unloaded(entry_file_path.to_owned());
    whole_program.files.insert(String::from(filename), x);

    while true {
        let unloaded_file = whole_program
            .files
            .iter_mut()
            .find(|(_, v)| v.ast.is_none());
        if unloaded_file.is_none() {
            // All files are loaded, nothing to do in the loop anymore
            break;
        }
        let unloaded_file = unloaded_file.unwrap().1;

        generate_ast(unloaded_file);
        if unloaded_file.ast.is_none() {
            // Error occured, no AST was generated
            return;
        }
    }
}
