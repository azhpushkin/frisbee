use crate::ast::*;
use crate::loader::WholeProgram;
use std::collections::HashMap;

mod execution_env;
mod expressions;
mod module_types;
mod operators;
mod statements;
mod tests;

pub fn perform_checks(wp: &WholeProgram) {
    for (file_name, file) in wp.files.iter() {
        module_types::check_collision_of_imports_and_definitions_per_module(&file.ast);
        module_types::check_type_is_not_referring_self(&file.ast);
        module_types::check_imports_of_itself(file);
        module_types::check_imports_are_correct(&file.ast.imports, wp);
    }

    for (_, file) in wp.files.iter() {
        let mut env = execution_env::get_env_for_file(wp, file);

        for typedef in file.ast.types.values() {
            env.scope = Some(typedef.clone());
            env.variables_types = HashMap::new();
            for method in typedef.methods.values() {
                statements::check_statements(&method.statements, &env);
            }
        }

        for funcdef in file.ast.functions.values() {
            env.scope = None;
            env.variables_types = HashMap::new();
            statements::check_statements(&funcdef.statements, &env);
        }
    }
}
