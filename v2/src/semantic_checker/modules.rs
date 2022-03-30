use std::collections::HashMap;
use std::iter::FromIterator;

use super::semantic_error::{sem_err, SemanticError, SemanticResult};
use crate::ast::*;
use crate::loader::*;

pub struct ObjectSignature {
    pub module_path_alias: ModulePathAlias,
    pub name: String,
    pub is_active: bool,
    pub fields: HashMap<String, Type>,
    pub methods: HashMap<String, FunctionSignature>,
}

pub struct FunctionSignature {
    pub rettype: Type,
    pub args: HashMap<String, Type>,
}

// These are applicable for both Types and functions
pub type ObjectPath = (ModulePathAlias, String);
pub type FileObjectsMapping = HashMap<String, ObjectPath>;

pub struct FileMappings {
    pub typenames: FileObjectsMapping,
    pub functions: FileObjectsMapping,
}

pub fn get_typenames_mapping(file: &LoadedFile) -> SemanticResult<FileObjectsMapping> {
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

pub fn get_functions_mapping(file: &LoadedFile) -> SemanticResult<FileObjectsMapping> {
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

pub fn get_typenames_signatures(
    file: &LoadedFile,
    typenames_mapping: &FileObjectsMapping,
) -> SemanticResult<HashMap<ObjectPath, ObjectSignature>> {
    let mut signatures: HashMap<ObjectPath, ObjectSignature> = HashMap::new();

    for class_decl in file.ast.types.iter() {
        let object_path: ObjectPath = (file.module_path.alias(), class_decl.name.clone());
        if does_class_contains_itself(class_decl) {
            // This will result in memory layout recursion, if allowed
            return sem_err!(
                "Type {} in {:?} contains itself, not allowed!",
                class_decl.name,
                file.module_path.alias()
            );
        }

        let mut class_signature = ObjectSignature {
            module_path_alias: file.module_path.alias(),
            name: class_decl.name.clone(),
            is_active: class_decl.is_active,
            fields: typednameobjects_to_hashmap(&class_decl.fields, typenames_mapping)?,
            methods: HashMap::new(),
        };

        for method in class_decl.methods.iter() {
            let method_signature = FunctionSignature {
                rettype: annotate_type(&method.rettype, typenames_mapping)?,
                args: typednameobjects_to_hashmap(&method.args, typenames_mapping)?,
            };
            let prev = class_signature
                .methods
                .insert(method.name.clone(), method_signature);
            if prev.is_some() {
                return sem_err!(
                    "Redefinition of method {} in {:?}",
                    method.name,
                    object_path
                );
            }
        }

        signatures.insert(object_path, class_signature);
    }
    Ok(signatures)
}

pub fn get_functions_signatures(
    file: &LoadedFile,
    typenames_mapping: &FileObjectsMapping,
) -> SemanticResult<HashMap<ObjectPath, FunctionSignature>> {
    let mut signatures: HashMap<ObjectPath, FunctionSignature> = HashMap::new();

    for function_decl in file.ast.functions.iter() {
        let object_path: ObjectPath = (file.module_path.alias(), function_decl.name.clone());
        let signature = FunctionSignature {
            rettype: annotate_type(&function_decl.rettype, typenames_mapping)?,
            args: typednameobjects_to_hashmap(&function_decl.args, typenames_mapping)?,
        };
        signatures.insert(object_path, signature);
    }
    Ok(signatures)
}

fn annotate_type(t: &Type, typenames_mapping: &FileObjectsMapping) -> SemanticResult<Type> {
    let new_t = match t {
        Type::TypeInt => Type::TypeInt,
        Type::TypeFloat => Type::TypeFloat,
        Type::TypeNil => Type::TypeNil,
        Type::TypeBool => Type::TypeBool,
        Type::TypeString => Type::TypeString,

        Type::TypeList(t) => {
            Type::TypeList(Box::new(annotate_type(t.as_ref(), typenames_mapping)?))
        }
        Type::TypeMaybe(t) => {
            Type::TypeMaybe(Box::new(annotate_type(t.as_ref(), typenames_mapping)?))
        }
        Type::TypeTuple(ts) => {
            let ts_annotated: SemanticResult<Vec<Type>> = ts
                .iter()
                .map(|t| annotate_type(t, typenames_mapping))
                .collect();
            Type::TypeTuple(ts_annotated?)
        }

        Type::TypeIdent(s) => {
            let obj_path = typenames_mapping.get(s);
            if obj_path.is_none() {
                return sem_err!("Type {} is not defined in this scope!", s);
            } else {
                let (alias, name) = obj_path.unwrap();
                Type::TypeIdentQualified(alias.clone(), name.clone())
            }
        }
        Type::TypeIdentQualified(..) => panic!("Did not expected {:?}", t),
    };
    Ok(new_t)
}

fn typednameobjects_to_hashmap(
    items: &Vec<TypedNamedObject>,
    typenames_mapping: &FileObjectsMapping,
) -> SemanticResult<HashMap<String, Type>> {
    let annotated: SemanticResult<Vec<Type>> = items
        .iter()
        .map(|t| annotate_type(&t.objtype, typenames_mapping))
        .collect();
    let annotated = annotated?;
    Ok(HashMap::from_iter(
        items
            .iter()
            .enumerate()
            .map(|(i, t)| (t.name.clone(), annotated[i].clone())),
    ))
}

// pub fn get_module_types(file: &LoadedFile) -> HashMap<ObjectPath, ObjectSignature> {
//     for objtype in file.ast.types.iter() {
//         let object_path = (file.module_path.alias(), objtype.name.clone()),
//     }
// }

pub fn check_module_does_not_import_itself(file: &LoadedFile) -> SemanticResult<()> {
    for import in &file.ast.imports {
        if import.module_path == file.module_path {
            return sem_err!("Module {:?} is importing itself!", file.module_path.alias());
        }
    }
    Ok(())
}

fn does_type_contain_itself(field_type: &Type, type_name: &String) -> bool {
    match field_type {
        Type::TypeIdent(s) if s == type_name => true,
        Type::TypeTuple(v) => v.iter().any(|t| does_type_contain_itself(t, type_name)),
        _ => false,
    }
}

pub fn does_class_contains_itself(class_decl: &ClassDecl) -> bool {
    if class_decl.is_active {
        // Active Type is in fact reference, so it is fine to have active refer to itself
        return false;
    }
    let check_field =
        |field: &TypedNamedObject| does_type_contain_itself(&field.objtype, &class_decl.name);
    return class_decl.fields.iter().any(check_field);
}

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
