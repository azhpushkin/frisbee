use crate::ast::*;
use crate::loader::{LoadedFile, WholeProgram};

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

pub fn check_type_is_not_referring_self(ast: &FileAst) {
    let types_decl = ast.actives.iter().chain(ast.classes.iter());
    for (type_name, object_decl) in types_decl {
        for field in object_decl.fields.values() {
            let TypedNamedObject { typename: field_type, .. } = field;
            match field_type {
                Type::TypeIdent(s) if s == type_name => {
                    panic!("Type {} references itself", type_name)
                }
                _ => (),
            }
        }
    }
}

pub fn perform_checks(wp: &WholeProgram) {
    for (_, file) in &wp.files {
        check_collision_of_imports_and_definitions_per_module(&file.ast);
        check_type_is_not_referring_self(&file.ast);
    }
}
