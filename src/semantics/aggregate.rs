use std::collections::HashMap;

use crate::ast::{ClassDecl, FunctionDecl, TypedNamedObject};
use crate::loader::{ModuleAlias, WholeProgram};
use crate::types::Type;

use super::annotations::{annotate_type, annotate_typednamed_vec, CustomType, TypedFields};
use super::errors::{top_level_with_module, SemanticResultWithModule};
use super::light_ast::LStatement;
use super::resolvers::NameResolver;
use super::symbols::{SymbolFunc, SymbolType};

#[derive(Debug)]
pub struct ProgramAggregate {
    pub types: HashMap<SymbolType, CustomType>,
    pub functions: HashMap<SymbolFunc, RawFunction>,
    pub entry: SymbolFunc,
}

#[derive(Debug)]
pub struct RawFunction {
    pub name: SymbolFunc,
    pub return_type: Type,
    pub args: TypedFields,
    pub body: Vec<LStatement>,

    pub short_name: String,
    pub method_of: Option<SymbolType>,
    pub defined_at: ModuleAlias,
}

/// Creates basic aggregate, that contains only types
pub fn create_basic_aggregate(
    wp: &WholeProgram,
    resolver: &NameResolver,
) -> SemanticResultWithModule<ProgramAggregate> {
    let mut aggregate: ProgramAggregate = ProgramAggregate {
        types: HashMap::new(),
        functions: HashMap::new(),
        entry: SymbolFunc::new(&wp.main_module, &String::from("main")),
    };

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(&file_alias);

        let field_type_error = |err: String, class: &ClassDecl| {
            top_level_with_module!(
                file_alias,
                "Error in {} class field types: {}",
                class.name,
                err
            )
        };

        for class_decl in file.ast.types.iter() {
            let full_name = SymbolType::new(file_alias, &class_decl.name);
            aggregate.types.insert(
                full_name.clone(),
                CustomType {
                    name: full_name,
                    is_active: class_decl.is_active,
                    fields: annotate_typednamed_vec(&class_decl.fields, &file_resolver)
                        .or_else(|err| field_type_error(err, &class_decl))?,
                },
            );
        }
    }

    Ok(aggregate)
}

pub fn fill_aggregate_with_funcs<'a>(
    wp: &'a WholeProgram,
    aggregate: &mut ProgramAggregate,
    resolver: &NameResolver,
) -> SemanticResultWithModule<HashMap<SymbolFunc, &'a FunctionDecl>> {
    let mut mapping_to_og_funcs = HashMap::new();

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(&file_alias);

        let return_type_err = |funcname, e| {
            top_level_with_module!(
                file_alias,
                "Bad return type of function {}: {}",
                funcname,
                e
            )
        };
        let args_type_err = |funcname, e| {
            top_level_with_module!(
                file_alias,
                "Bad argument type in function {}: {}",
                funcname,
                e
            )
        };

        let get_return_type = |t: &_| match t {
            None => Ok(Type::Tuple(vec![])),
            Some(t) => annotate_type(t, &file_resolver),
        };

        for class_decl in file.ast.types.iter() {
            let type_full_name = SymbolType::new(file_alias, &class_decl.name);

            for method in class_decl.methods.iter() {
                let method_full_name = type_full_name.method(&method.name);
                if aggregate.functions.contains_key(&method_full_name) {
                    return top_level_with_module!(
                        file_alias,
                        "Method {} defined twice in {}",
                        method.name,
                        class_decl.name
                    );
                }

                let mut args = method.args.clone();
                if method.name != class_decl.name {
                    args.insert(
                        0,
                        TypedNamedObject {
                            name: "this".to_string(),
                            typename: Type::Ident(class_decl.name.clone()),
                        },
                    );
                };
                aggregate.functions.insert(
                    method_full_name.clone(),
                    RawFunction {
                        name: method_full_name.clone(),
                        return_type: get_return_type(&method.rettype)
                            .or_else(|e| return_type_err(&method.name, e))?,
                        args: annotate_typednamed_vec(&args, &file_resolver)
                            .or_else(|e| args_type_err(&method.name, e))?,
                        body: vec![],
                        short_name: method.name.clone(),
                        method_of: Some(type_full_name.clone()),
                        defined_at: file_alias.clone(),
                    },
                );
                mapping_to_og_funcs.insert(method_full_name.clone(), method);
            }
        }

        for function_decl in file.ast.functions.iter() {
            let full_name = SymbolFunc::new(file_alias, &function_decl.name);

            // No checks for function redefinition here because resolver already does one
            aggregate.functions.insert(
                full_name.clone(),
                RawFunction {
                    name: full_name.clone(),
                    return_type: get_return_type(&function_decl.rettype)
                        .or_else(|e| return_type_err(&function_decl.name, e))?,
                    args: annotate_typednamed_vec(&function_decl.args, &file_resolver)
                        .or_else(|e| args_type_err(&function_decl.name, e))?,
                    body: vec![],
                    short_name: function_decl.name.clone(),
                    method_of: None,
                    defined_at: file_alias.clone(),
                },
            );
            mapping_to_og_funcs.insert(full_name.clone(), function_decl);
        }
    }

    if let Some(raw_entry) = aggregate.functions.get(&aggregate.entry) {
        if raw_entry.return_type != Type::Tuple(vec![]) {
            return top_level_with_module!(
                wp.main_module,
                "Entry function {} must return void, but it returns {}",
                aggregate.entry,
                raw_entry.return_type,
            );
        }
    } else {
        return top_level_with_module!(
            wp.main_module,
            "Entry function {} not found",
            aggregate.entry
        );
    }

    Ok(mapping_to_og_funcs)
}
