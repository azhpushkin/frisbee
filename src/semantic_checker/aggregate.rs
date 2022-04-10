use std::collections::HashMap;

use crate::ast::ModulePathAlias;
use crate::loader::WholeProgram;

use super::real_ast::RStatement;
use super::real_type::{type_vec_to_typed_fields, CustomType, RType, TypedFields};
use super::resolvers::{compile_name, NameResolver};

#[derive(Debug)]
pub struct ProgramAggregate {
    pub types: HashMap<String, CustomType>,
    pub functions: HashMap<String, RFunction>,
}

#[derive(Debug)]
pub struct RFunction {
    pub name: String,
    pub return_type: RType,
    pub args: TypedFields,
    pub body: Vec<RStatement>,
}

pub fn create_basic_aggregate(wp: &WholeProgram, resolver: &NameResolver) -> ProgramAggregate {
    let mut aggregate: ProgramAggregate =
        ProgramAggregate { types: HashMap::new(), functions: HashMap::new() };

    for (file_alias, file) in wp.files.iter() {
        let file_resolver = resolver.get_typenames_resolver(file_alias.clone());

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

// pub fn check_class_does_not_contains_itself(class_decl: &ClassDecl) -> SemanticResult<()> {
//     if class_decl.is_active {
//         // Active Type is in fact reference, so it is fine to have active refer to itself
//         return Ok(());
//     }
//     let check_field =
//         |field: &TypedNamedObject| does_type_contain_itself(&field.typename, &class_decl.name);
//     if class_decl.fields.iter().any(check_field) {
//         return sem_err!("Type {} in contains itself, not allowed!", class_decl.name,);
//     } else {
//         Ok(())
//     }
// }
