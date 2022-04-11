use std::collections::HashMap;

use crate::ast::{FunctionDecl, ModulePathAlias};
use crate::loader::WholeProgram;

use super::light_ast::LStatement;
use super::real_type::{type_to_real, type_vec_to_typed_fields, CustomType, RType, TypedFields};
use super::resolvers::{compile_method_name, compile_name, NameResolver, Symbol};

#[derive(Debug)]
pub struct ProgramAggregate {
    pub types: HashMap<Symbol, CustomType>,
    pub functions: HashMap<Symbol, RawFunction>,
}

#[derive(Debug)]
pub struct RawFunction {
    pub name: Symbol,
    pub return_type: Option<RType>,
    pub args: TypedFields,
    pub body: Vec<LStatement>,

    pub short_name: String,
    pub method_of: Option<Symbol>,
    pub defined_at: ModulePathAlias,
}

/// Creates basic aggregate, that contains only types
pub fn create_basic_aggregate(wp: &WholeProgram, resolver: &NameResolver) -> ProgramAggregate {
    let mut aggregate: ProgramAggregate =
        ProgramAggregate { types: HashMap::new(), functions: HashMap::new() };

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(&file_alias);

        for class_decl in file.ast.types.iter() {
            let full_name = compile_name(file_alias, &class_decl.name);
            aggregate.types.insert(
                full_name.clone(),
                CustomType {
                    name: full_name,
                    is_active: class_decl.is_active,
                    fields: type_vec_to_typed_fields(&class_decl.fields, &file_resolver),
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
) -> HashMap<Symbol, &'a FunctionDecl> {
    let mapping_to_og_funcs = HashMap::new();

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(&file_alias);

        let get_return_type = |t: &_| match t {
            None => None,
            Some(t) => Some(type_to_real(t, &file_resolver)),
        };

        for class_decl in file.ast.types.iter() {
            let type_full_name = compile_name(file_alias, &class_decl.name);

            for method in class_decl.methods.iter() {
                let method_full_name =
                    compile_method_name(file_alias, &class_decl.name, &method.name);
                if aggregate.functions.contains_key(&method_full_name) {
                    panic!(
                        "Method {} defined twice in {}.{}",
                        method.name, file_alias.0, class_decl.name
                    );
                }

                aggregate.functions.insert(
                    method_full_name.clone(),
                    RawFunction {
                        name: method_full_name,
                        return_type: get_return_type(&method.rettype),
                        args: type_vec_to_typed_fields(&method.args, &file_resolver),
                        body: vec![],
                        short_name: method.name.clone(),
                        method_of: Some(type_full_name.clone()),
                        defined_at: file_alias.clone(),
                    },
                );
                mapping_to_og_funcs.insert(method_full_name, method);
            }
        }

        for function_decl in file.ast.functions.iter() {
            let full_name = compile_name(file_alias, &function_decl.name);

            // No checks for function redefinition here because resolver already does one
            aggregate.functions.insert(
                full_name.clone(),
                RawFunction {
                    name: full_name,
                    return_type: get_return_type(&function_decl.rettype),
                    args: type_vec_to_typed_fields(&function_decl.args, &file_resolver),
                    body: vec![],
                    short_name: function_decl.name.clone(),
                    method_of: None,
                    defined_at: file_alias.clone(),
                },
            );
            mapping_to_og_funcs.insert(full_name, function_decl);
        }
    }

    mapping_to_og_funcs
}
