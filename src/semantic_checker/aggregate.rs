use std::collections::HashMap;

use crate::ast::ModulePathAlias;
use crate::loader::WholeProgram;

use super::real_ast::RStatement;
use super::real_type::{CustomType, RType, TypedFields, type_vec_to_typed_fields};

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


pub fn compile_name(alias: &ModulePathAlias, name: &String) -> String {
    format!("{}::{}", alias.0, name)
}
pub fn compile_method(alias: &ModulePathAlias, typename: &String, method: &String) -> String {
    format!("{}::{}::{}", alias.0, typename, method)
}


pub fn create_basic_aggregate(wp: &WholeProgram) -> ProgramAggregate {
    let mut aggregate: ProgramAggregate =
        ProgramAggregate { types: HashMap::new(), functions: HashMap::new() };

    for (file_alias, file) in wp.files.iter() {
        for class_decl in file.ast.types {
            let full_name = compile_name(file_alias, &class_decl.name);
            aggregate.types.insert(
                full_name,
                CustomType { name: full_name, is_active: class_decl.is_active, fields: type_vec_to_typed_fields(&class_decl.fields) }
            );
        }
    }

    aggregate
}
