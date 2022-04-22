use std::collections::HashMap;

use crate::ast::{FunctionDecl, TypedNamedObject};
use crate::loader::{WholeProgram, ModuleAlias};
use crate::types::Type;

use super::annotations::{annotate_type, annotate_typednamed_vec, CustomType, TypedFields};
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
pub fn create_basic_aggregate(wp: &WholeProgram, resolver: &NameResolver) -> ProgramAggregate {
    let mut aggregate: ProgramAggregate = ProgramAggregate {
        types: HashMap::new(),
        functions: HashMap::new(),
        entry: SymbolFunc::new(&wp.main_module, &String::from("main")),
    };

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(&file_alias);

        for class_decl in file.ast.types.iter() {
            let full_name = SymbolType::new(file_alias, &class_decl.name);
            aggregate.types.insert(
                full_name.clone(),
                CustomType {
                    name: full_name,
                    is_active: class_decl.is_active,
                    fields: annotate_typednamed_vec(&class_decl.fields, &file_resolver),
                },
            );
        }
    }

    aggregate
}

pub fn fill_aggregate_with_funcs<'a>(
    wp: &'a WholeProgram,
    aggregate: &mut ProgramAggregate,
    resolver: &NameResolver,
) -> HashMap<SymbolFunc, &'a FunctionDecl> {
    let mut mapping_to_og_funcs = HashMap::new();

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(&file_alias);

        let get_return_type = |t: &_| match t {
            None => Type::Tuple(vec![]),
            Some(t) => annotate_type(t, &file_resolver),
        };

        for class_decl in file.ast.types.iter() {
            let type_full_name = SymbolType::new(file_alias, &class_decl.name);

            for method in class_decl.methods.iter() {
                let method_full_name = type_full_name.method(&method.name);
                if aggregate.functions.contains_key(&method_full_name) {
                    panic!(
                        "Method {} defined twice in {}.{}",
                        method.name, file_alias, class_decl.name
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
                        return_type: get_return_type(&method.rettype),
                        args: annotate_typednamed_vec(&method.args, &file_resolver),
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
                    return_type: get_return_type(&function_decl.rettype),
                    args: annotate_typednamed_vec(&function_decl.args, &file_resolver),
                    body: vec![],
                    short_name: function_decl.name.clone(),
                    method_of: None,
                    defined_at: file_alias.clone(),
                },
            );
            mapping_to_og_funcs.insert(full_name.clone(), function_decl);
        }
    }

    if !aggregate.functions.contains_key(&aggregate.entry) {
        panic!("Main function {:?} not defined!", aggregate.entry);
    }

    mapping_to_og_funcs
}
