use crate::{ast::*, loader::LoadedFile};

pub fn check_collision_of_imports_and_definitions_per_module(ast: &FileAst) {
    for import in &ast.imports {
        for typename in &import.typenames {
            if ast.types.contains_key(typename) {
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

fn is_type_referring_itself(type_name: &String, field_type: &Type) -> bool {
    match field_type {
        Type::TypeIdent(s) if s == type_name => true,
        Type::TypeTuple(v) => v.iter().any(|t| is_type_referring_itself(type_name, t)),
        _ => false,
    }
}

pub fn check_type_is_not_referring_self(ast: &FileAst) {
    for (type_name, object_decl) in ast.types.iter() {
        for field in object_decl.fields.values() {
            let TypedNamedObject { typename: field_type, .. } = field;
            if is_type_referring_itself(type_name, field_type) {
                panic!("Type {} references itself", type_name)
            }
        }
    }
}

pub fn check_imports_of_itself(file: &LoadedFile) {
    for import in &file.ast.imports {
        if import.module_path == file.module_path {
            panic!("Importing self in {:?} is meaningless", import);
        }
    }
}
