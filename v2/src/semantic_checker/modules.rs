use std::collections::HashMap;
use std::iter::FromIterator;

use crate::ast::*;
use crate::loader::*;

use super::semantic_error::{sem_err, SemanticError, SemanticResult};
use super::symbols::*;

pub fn get_typenames_mapping(file: &LoadedFile) -> SemanticResult<SymbolOriginsMapping> {
    let file_alias = file.module_path.alias();
    let mut mapping: SymbolOriginsMapping = HashMap::new();

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

    for (module_alias, typename) in defined_types.chain(imported_types) {
        if mapping.contains_key(&typename) {
            return sem_err!(
                "Type {} introduced several times in module {:?}",
                typename,
                file_alias
            );
        }
        mapping.insert(
            typename.clone(),
            SymbolOrigin { module: module_alias, name: typename },
        );
    }

    Ok(mapping)
}

pub fn get_functions_mapping(file: &LoadedFile) -> SemanticResult<SymbolOriginsMapping> {
    let file_alias = file.module_path.alias();
    let mut mapping: SymbolOriginsMapping = HashMap::new();

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

    for (module_alias, funcname) in defined_types.chain(imported_types) {
        if mapping.contains_key(&funcname) {
            return sem_err!(
                "Function {} introduced several times in module {:?}",
                funcname,
                file_alias
            );
        }
        mapping.insert(
            funcname.clone(),
            SymbolOrigin { module: module_alias, name: funcname },
        );
    }

    Ok(mapping)
}

pub fn get_typenames_signatures(
    file: &LoadedFile,
    typenames_mapping: &SymbolOriginsMapping,
) -> SemanticResult<HashMap<SymbolOrigin, ClassSignature>> {
    let mut signatures: HashMap<SymbolOrigin, ClassSignature> = HashMap::new();

    for class_decl in file.ast.types.iter() {
        let symbol_origin =
            SymbolOrigin { module: file.module_path.alias(), name: class_decl.name.clone() };
        if does_class_contains_itself(class_decl) {
            // This will result in memory layout recursion, if allowed
            return sem_err!(
                "Type {} in {:?} contains itself, not allowed!",
                class_decl.name,
                file.module_path.alias()
            );
        }

        let mut class_signature = ClassSignature {
            module_path_alias: file.module_path.alias(),
            name: class_decl.name.clone(),
            is_active: class_decl.is_active,
            fields: (typednameobjects_to_func_args(&class_decl.fields, typenames_mapping)?
                .into_iter()
                .collect()),
            methods: HashMap::new(),
        };

        for method in class_decl.methods.iter() {
            let method_signature = FunctionSignature {
                rettype: annotate_type(&method.rettype, typenames_mapping)?,
                args: typednameobjects_to_func_args(&method.args, typenames_mapping)?,
            };
            let prev = class_signature
                .methods
                .insert(method.name.clone(), method_signature);
            if prev.is_some() {
                return sem_err!(
                    "Redefinition of method {} in {:?}",
                    method.name,
                    symbol_origin
                );
            }
        }

        // If no constructor is mentioned - then add default one for typecheck
        if !class_signature.methods.contains_key(&class_decl.name) {
            let constructor = FunctionSignature {
                rettype: Type::TypeIdentQualified(
                    file.module_path.alias(),
                    class_decl.name.clone(),
                ),
                args: typednameobjects_to_func_args(&class_decl.fields, typenames_mapping)?,
            };
            class_signature
                .methods
                .insert(class_decl.name.clone(), constructor);
        }

        signatures.insert(symbol_origin, class_signature);
    }
    Ok(signatures)
}

pub fn get_functions_signatures(
    file: &LoadedFile,
    typenames_mapping: &SymbolOriginsMapping,
) -> SemanticResult<HashMap<SymbolOrigin, FunctionSignature>> {
    let mut signatures: HashMap<SymbolOrigin, FunctionSignature> = HashMap::new();

    for function_decl in file.ast.functions.iter() {
        let symbol_origin =
            SymbolOrigin { module: file.module_path.alias(), name: function_decl.name.clone() };
        let signature = FunctionSignature {
            rettype: annotate_type(&function_decl.rettype, typenames_mapping)?,
            args: typednameobjects_to_func_args(&function_decl.args, typenames_mapping)?,
        };
        signatures.insert(symbol_origin, signature);
    }
    Ok(signatures)
}

pub fn annotate_type(t: &Type, typenames_mapping: &SymbolOriginsMapping) -> SemanticResult<Type> {
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
            let symbol_origin = typenames_mapping.get(s);
            if let Some(symbol_origin) = symbol_origin {
                Type::TypeIdentQualified(symbol_origin.module.clone(), symbol_origin.name.clone())
            } else {
                return sem_err!("Unknown type {}", s);
            }
        }
        Type::TypeIdentQualified(..) => panic!("Did not expected {:?}", t),
        Type::TypeAnonymous => panic!("Did not expected {:?}", t),
    };
    Ok(new_t)
}

fn typednameobjects_to_func_args(
    items: &Vec<TypedNamedObject>,
    typenames_mapping: &SymbolOriginsMapping,
) -> SemanticResult<Vec<(String, Type)>> {
    let mut res: Vec<(String, Type)> = vec![];
    res.reserve_exact(items.len());
    for item in items {
        res.push((
            item.name.clone(),
            annotate_type(&item.typename, typenames_mapping)?,
        ));
    }
    return Ok(res);
}

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
        |field: &TypedNamedObject| does_type_contain_itself(&field.typename, &class_decl.name);
    return class_decl.fields.iter().any(check_field);
}
