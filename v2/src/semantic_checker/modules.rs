use std::collections::HashMap;
use std::iter::Iterator;

use crate::ast::*;
use crate::loader::*;

use super::semantic_error::{sem_err, SemanticResult};
use super::symbols::*;

fn typed_named_to_vec(items: &Vec<TypedNamedObject>) -> Vec<(String, Type)> {
    items.iter().map(|o| (o.name.clone(), o.typename.clone())).collect()
}

pub fn get_typenames_origins(file: &LoadedFile) -> SemanticResult<SymbolOriginsMapping> {
    let file_alias = file.module_path.alias();
    let mut mapping: SymbolOriginsMapping = HashMap::new();

    let defined_types = file.ast.types.iter().map(|d| (file.module_path.alias(), d.name.clone()));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.typenames.iter().map(move |typename| (i.module_path.alias(), typename.clone()))
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

    let defined_types =
        file.ast.functions.iter().map(|f| (file.module_path.alias(), f.name.clone()));

    let imported_types = file.ast.imports.iter().flat_map(|i| {
        i.functions.iter().map(move |funcname| (i.module_path.alias(), funcname.clone()))
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

pub fn get_typenames_signatures(file: &LoadedFile) -> HashMap<SymbolOrigin, ClassSignature> {
    let mut signatures: HashMap<SymbolOrigin, ClassSignature> = HashMap::new();

    for class_decl in file.ast.types.iter() {
        let class_symbol_origin =
            SymbolOrigin { module: file.module_path.alias(), name: class_decl.name.clone() };

        let mut class_signature = ClassSignature {
            module_path_alias: file.module_path.alias(),
            name: class_decl.name.clone(),
            is_active: class_decl.is_active,
            fields: typed_named_to_vec(&class_decl.fields).into_iter().collect(),
            methods: HashMap::new(),
        };

        for method in class_decl.methods.iter() {
            let method_signature = FunctionSignature {
                rettype: method.rettype.clone(),
                args: typed_named_to_vec(&method.args),
            };
            class_signature.methods.insert(method.name.clone(), method_signature);
        }

        signatures.insert(class_symbol_origin, class_signature);
    }
    signatures
}

pub fn get_functions_signatures(file: &LoadedFile) -> HashMap<SymbolOrigin, FunctionSignature> {
    let mut signatures: HashMap<SymbolOrigin, FunctionSignature> = HashMap::new();

    for function_decl in file.ast.functions.iter() {
        let symbol_origin =
            SymbolOrigin { module: file.module_path.alias(), name: function_decl.name.clone() };
        let signature = FunctionSignature {
            rettype: function_decl.rettype.clone(),
            args: typed_named_to_vec(&function_decl.args),
        };
        signatures.insert(symbol_origin, signature);
    }
    signatures
}

pub fn check_module_does_not_import_itself(file: &LoadedFile) -> SemanticResult<()> {
    for import in &file.ast.imports {
        if import.module_path == file.module_path {
            return sem_err!("Module {:?} is importing itself!", file.module_path.alias());
        }
    }
    Ok(())
}

pub fn check_class_does_not_contains_itself(class_decl: &ClassDecl) -> SemanticResult<()> {
    if class_decl.is_active {
        // Active Type is in fact reference, so it is fine to have active refer to itself
        return Ok(());
    }
    let check_field =
        |field: &TypedNamedObject| does_type_contain_itself(&field.typename, &class_decl.name);
    if class_decl.fields.iter().any(check_field) {
        return sem_err!("Type {} in contains itself, not allowed!", class_decl.name,);
    } else {
        Ok(())
    }
}

pub fn check_class_has_no_duplicated_methods(class_decl: &ClassDecl) -> SemanticResult<()> {
    for method in class_decl.methods.iter() {
        // TODO: this might be optimized to remove O(n^2) complexity
        let same_named_methods =
            class_decl.methods.iter().filter(|m| m.name == method.name).count();
        if same_named_methods > 1 {
            return sem_err!(
                "Class {} has more than one method named {}",
                class_decl.name,
                method.name
            );
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
