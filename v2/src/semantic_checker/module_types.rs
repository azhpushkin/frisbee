use std::collections::HashMap;

use super::semantic_error::{sem_err, SemanticError, SemanticResult};
use crate::ast::*;
use crate::loader::*;

pub struct ObjectSignature {
    module_path_alias: ModulePathAlias,
    name: String,
    is_active: bool,
    fields: HashMap<String, Type>,
    methods: HashMap<String, FunctionSignature>,
}

pub struct FunctionSignature {
    rettype: Type,
    args: HashMap<String, Type>,
}

// These are applicable for both Types and functions
pub type ObjectPath = (ModulePathAlias, String);
pub type FileObjectsMapping = HashMap<String, ObjectPath>;

pub fn get_typenames_mapping(file: &LoadedFile) -> Result<FileObjectsMapping, SemanticError> {
    let file_alias = file.module_path.alias();
    let mut mapping: FileObjectsMapping = HashMap::new();

    let defined_types = file
        .ast
        .types
        .iter()
        .map(|d| (file.module_path.alias(), d.name.clone()));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.typenames
            .iter()
            .map(move |typename| (i.module_path.alias(), typename.clone()))
    });

    for obj_path in defined_types.chain(imported_types) {
        if mapping.contains_key(&obj_path.1) {
            return sem_err!(
                "Type {} introduced several times in module {:?}",
                obj_path.1,
                file_alias
            );
        }
        mapping.insert(obj_path.1.clone(), obj_path);
    }

    Ok(mapping)
}

pub fn get_functions_mapping(file: &LoadedFile) -> Result<FileObjectsMapping, SemanticError> {
    let file_alias = file.module_path.alias();
    let mut mapping: FileObjectsMapping = HashMap::new();

    let defined_types = file
        .ast
        .functions
        .iter()
        .map(|f| (file.module_path.alias(), f.name.clone()));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.functions
            .iter()
            .map(move |funcname| (i.module_path.alias(), funcname.clone()))
    });

    for obj_path in defined_types.chain(imported_types) {
        if mapping.contains_key(&obj_path.1) {
            return sem_err!(
                "Function {} introduced several times in module {:?}",
                obj_path.1,
                file_alias
            );
        }
        mapping.insert(obj_path.1.clone(), obj_path);
    }

    Ok(mapping)
}

// fn annotate_type(t: Type) -> Type {
//     match t {
//         Type::TypeInt => Type::TypeInt,
//         Type::TypeFloat => Type::TypeFloat,
//         Type::TypeNil => Type::TypeNil,
//         Type::TypeBool => Type::TypeBool,
//         Type::TypeString => Type::TypeString,

//         Type::TypeList{..} => Type::TypeList(),
//         Type::TypeTuple{..} => Type::TypeTuple(),
//         Type::TypeMaybe{..} => Type::TypeMaybe(),

//         Type::TypeIdent{..} => Type::TypeIdent(),
//     }

// }

// pub fn get_module_types(file: &LoadedFile) -> HashMap<ObjectPath, ObjectSignature> {
//     for objtype in file.ast.types.iter() {
//         let object_path = (file.module_path.alias(), objtype.name.clone()),
//     }
// }

pub fn check_imports_of_itself(file: &LoadedFile) -> SemanticResult {
    for import in &file.ast.imports {
        if import.module_path == file.module_path {
            return sem_err!("Module {:?} is importing itself!", file.module_path.alias());
        }
    }
    Ok(())
}

// fn is_type_referring_itself(type_name: &String, field_type: &Type) -> bool {
//     match field_type {
//         Type::TypeIdent(s) if s == type_name => true,
//         Type::TypeTuple(v) => v.iter().any(|t| is_type_referring_itself(type_name, t)),
//         _ => false,
//     }
// }

// pub fn check_type_is_not_referring_self(ast: &FileAst) {
//     for (type_name, object_decl) in ast.types.iter() {
//         for field in object_decl.fields.values() {
//             let TypedNamedObject { typename: field_type, .. } = field;
//             if is_type_referring_itself(type_name, field_type) {
//                 panic!("Type {} references itself", type_name)
//             }
//         }
//     }
// }

// pub fn check_imports_are_correct(imports: &Vec<ImportDecl>, wp: &WholeProgram) {
//     for import in imports {
//         let imported_module = wp.files.get(&import.module_path.alias());
//         let module_ast = &imported_module.unwrap().ast;

//         for function in &import.functions {
//             if !module_ast.functions.contains_key(function) {
//                 panic!(
//                     "Import {:?} refers to missing function {}",
//                     import, function
//                 );
//             }
//         }

//         for typename in &import.typenames {
//             if !module_ast.types.contains_key(typename) {
//                 panic!(
//                     "Import {:?} refers to missing function {}",
//                     import, typename
//                 );
//             }
//         }
//     }
// }
