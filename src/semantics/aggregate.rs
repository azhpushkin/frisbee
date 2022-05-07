use std::collections::HashMap;

use crate::alias::ModuleAlias;
use crate::ast::parsed::{ClassDecl, FileAst, FunctionDecl, TypedItem};
use crate::ast::verified::{CustomType, RawFunction, TypedFields};
use crate::symbols::{SymbolFunc, SymbolType, MAIN_FUNCTION_NAME};
use crate::types::{verify_parsed_type, Type};

use super::errors::{top_level_with_module, SemanticError, SemanticErrorWithModule};
use super::resolvers::{NameResolver, SymbolResolver};
use super::std_definitions::is_std_function;

#[derive(Debug)]
pub struct ProgramAggregate {
    pub types: HashMap<SymbolType, CustomType>,
    pub functions: HashMap<SymbolFunc, RawFunction>,
    pub entry: SymbolFunc,
}

/// Creates basic aggregate, that contains only types
pub fn create_basic_aggregate(
    modules: &[(&ModuleAlias, &FileAst)],
    entry_module: &ModuleAlias,
    resolver: &NameResolver,
) -> Result<ProgramAggregate, SemanticErrorWithModule> {
    let mut aggregate: ProgramAggregate = ProgramAggregate {
        types: HashMap::new(),
        functions: HashMap::new(),
        entry: SymbolFunc::new(entry_module, MAIN_FUNCTION_NAME),
    };

    for (alias, file_ast) in modules.iter() {
        let file_resolver = resolver.get_typenames_resolver(alias);

        let field_type_error = |err: String, class: &ClassDecl| {
            top_level_with_module!(
                *alias,
                class,
                "Error in {} class field types: {}",
                class.name,
                err
            )
        };

        for class_decl in file_ast.types.iter() {
            let full_name = SymbolType::new(alias, &class_decl.name);
            aggregate.types.insert(
                full_name.clone(),
                CustomType {
                    name: full_name,
                    is_active: class_decl.is_active,
                    fields: annotate_typednamed_vec(&class_decl.fields, &file_resolver)
                        .or_else(|err| field_type_error(err, class_decl))?,
                },
            );
        }

        if *alias == entry_module {
            check_entry_module_has_main(*alias, *file_ast)?;
        }
    }

    Ok(aggregate)
}

fn check_entry_module_has_main(
    main_module: &ModuleAlias,
    file_ast: &FileAst,
) -> Result<(), SemanticErrorWithModule> {
    let main_function_decl = file_ast.functions.iter().find(|func| func.name == MAIN_FUNCTION_NAME);

    if let Some(main_function_decl) = main_function_decl {
        if let Some(return_type) = &main_function_decl.rettype {
            return top_level_with_module!(
                main_module,
                main_function_decl,
                "Entry function `{}` must return void, but it returns {}",
                MAIN_FUNCTION_NAME,
                return_type,
            );
        }
    } else {
        return Err(SemanticErrorWithModule {
            module: main_module.clone(),
            error: SemanticError::TopLevelError {
                pos: 0,
                message: format!("Entry function `{}` not found", MAIN_FUNCTION_NAME),
            },
        });
    }
    Ok(())
}

pub fn fill_aggregate_with_funcs<'a>(
    modules: &[(&ModuleAlias, &'a FileAst)],
    aggregate: &mut ProgramAggregate,
    resolver: &NameResolver,
) -> Result<HashMap<SymbolFunc, &'a FunctionDecl>, SemanticErrorWithModule> {
    let mut mapping_to_og_funcs = HashMap::new();

    for (alias, file_ast) in modules.iter() {
        let file_resolver = resolver.get_typenames_resolver(alias);

        let return_type_err = |f: &FunctionDecl, e| {
            top_level_with_module!(*alias, f, "Bad return type of function {}: {}", f.name, e)
        };
        let args_type_err = |f: &FunctionDecl, e| {
            top_level_with_module!(*alias, f, "Bad argument type in function {}: {}", f.name, e)
        };

        let get_return_type = |t: &_| match t {
            None => Ok(Type::Tuple(vec![])),
            Some(t) => verify_parsed_type(t, &file_resolver),
        };

        for class_decl in file_ast.types.iter() {
            let type_full_name = SymbolType::new(alias, &class_decl.name);

            for method in class_decl.methods.iter() {
                let method_full_name = type_full_name.method(&method.name);
                if aggregate.functions.contains_key(&method_full_name) {
                    return top_level_with_module!(
                        *alias,
                        method,
                        "Method `{}` defined more than once in `{}`",
                        method.name,
                        class_decl.name
                    );
                }

                let mut args = method.args.clone();
                if method.name != class_decl.name {
                    args.insert(
                        0,
                        TypedItem {
                            name: "this".to_string(),
                            typename: Type::Custom(class_decl.name.clone()),
                        },
                    );
                };
                aggregate.functions.insert(
                    method_full_name.clone(),
                    RawFunction {
                        name: method_full_name.clone(),
                        return_type: get_return_type(&method.rettype)
                            .or_else(|e| return_type_err(method, e))?,
                        args: annotate_typednamed_vec(&args, &file_resolver)
                            .or_else(|e| args_type_err(method, e))?,
                        body: vec![],
                        locals: vec![],
                        short_name: method.name.clone(),
                        method_of: Some(type_full_name.clone()),
                        is_constructor: method.name == class_decl.name,
                        defined_at: (*alias).clone(),
                    },
                );
                mapping_to_og_funcs.insert(method_full_name.clone(), method);
            }
        }

        for function_decl in file_ast.functions.iter() {
            if is_std_function(&function_decl.name) {
                return top_level_with_module!(
                    *alias,
                    function_decl,
                    "Name {} is reserved by std function",
                    function_decl.name
                );
            }

            let full_name = SymbolFunc::new(alias, &function_decl.name);

            // No checks for function redefinition here because resolver already does one
            aggregate.functions.insert(
                full_name.clone(),
                RawFunction {
                    name: full_name.clone(),
                    return_type: get_return_type(&function_decl.rettype)
                        .or_else(|e| return_type_err(function_decl, e))?,
                    args: annotate_typednamed_vec(&function_decl.args, &file_resolver)
                        .or_else(|e| args_type_err(function_decl, e))?,
                    body: vec![],
                    locals: vec![],
                    short_name: function_decl.name.clone(),
                    method_of: None,
                    is_constructor: false,
                    defined_at: (*alias).clone(),
                },
            );
            mapping_to_og_funcs.insert(full_name.clone(), function_decl);
        }
    }

    Ok(mapping_to_og_funcs)
}

pub fn annotate_typednamed_vec(
    v: &[TypedItem],
    resolver: &SymbolResolver<SymbolType>,
) -> Result<TypedFields, String> {
    let mut typed_fields = TypedFields { names: HashMap::new(), types: vec![] };

    for (i, old_type) in v.iter().enumerate() {
        let real_type = verify_parsed_type(&old_type.typename, resolver)?;

        typed_fields.names.insert(i, old_type.name.clone());
        typed_fields.types.push(real_type);
    }
    Ok(typed_fields)
}
