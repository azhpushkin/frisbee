use crate::ast::*;
use crate::loader::{LoadedFile, WholeProgram};
use std::collections::HashMap;

fn check_collision_of_imports_and_definitions_per_module(ast: &FileAst) {
    for import in &ast.imports {
        for typename in &import.typenames {
            if ast.actives.contains_key(typename) || ast.classes.contains_key(typename) {
                panic!("Type {} is both imported and defined", typename);
            }
        }

        for funcname in &import.functions {
            if ast.functions.contains_key(funcname) {
                panic!("Function {} is both imported and defined", funcname);
            }
        }
    }
}

pub fn check_collision_of_imports_and_definitions(wp: &WholeProgram) {
    for (_, file) in &wp.files {
        check_collision_of_imports_and_definitions_per_module(&file.ast)
    }
}
