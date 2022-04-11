use std::collections::HashMap;

use crate::ast::ModulePathAlias;
use crate::loader::WholeProgram;

use super::light_ast::LStatement;
use super::real_type::{type_to_real, type_vec_to_typed_fields, CustomType, RType, TypedFields};
use super::resolvers::{compile_method_name, compile_name, NameResolver, Symbol};

#[derive(Debug)]
pub struct ProgramAggregate {
    pub types: HashMap<Symbol, CustomType>,
    pub functions: HashMap<Symbol, RFunction>,
}

#[derive(Debug)]
pub struct RFunction {
    pub name: Symbol,
    pub return_type: Option<RType>,
    pub args: TypedFields,
    pub body: Vec<LStatement>,

    pub method_of: Option<Symbol>,
    pub defined_at: ModulePathAlias,
}

/// Creates basic aggregate, that contains
/// * all types in final form
/// * all functions, with empty bodies
///
/// After this step we have to perform statements
/// analysis and fill in bodies of this functions
pub fn create_basic_aggregate(wp: &WholeProgram, resolver: &NameResolver) -> ProgramAggregate {
    let mut aggregate: ProgramAggregate =
        ProgramAggregate { types: HashMap::new(), functions: HashMap::new() };

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(&file_alias);
        let get_return_type = |t: &_| match t {
            None => None,
            Some(t) => Some(type_to_real(t, &file_resolver)),
        };

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
                    RFunction {
                        name: method_full_name,
                        return_type: get_return_type(&method.rettype),
                        args: type_vec_to_typed_fields(&method.args, &file_resolver),
                        body: vec![],
                        method_of: Some(full_name.clone()),
                        defined_at: file_alias.clone(),
                    },
                );
            }
        }

        for function_decl in file.ast.functions.iter() {
            let full_name = compile_name(file_alias, &function_decl.name);

            // No checks for function redefinition here because resolver already does one
            aggregate.functions.insert(
                full_name.clone(),
                RFunction {
                    name: full_name,
                    return_type: get_return_type(&function_decl.rettype),
                    args: type_vec_to_typed_fields(&function_decl.args, &file_resolver),
                    body: vec![],
                    method_of: None,
                    defined_at: file_alias.clone(),
                },
            );
        }
    }

    aggregate
}
