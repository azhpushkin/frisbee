use crate::loader::WholeProgram;
use std::collections::HashMap;

mod execution_env;
mod expressions;
mod module_types;
mod operators;
mod statements;
mod tests;

pub fn perform_checks(wp: &WholeProgram) {
    for (file_name, file) in &wp.files {
        module_types::check_collision_of_imports_and_definitions_per_module(&file.ast);
        module_types::check_type_is_not_referring_self(&file.ast);
        module_types::check_imports_of_itself(file);
        module_types::check_imports_are_correct(&file.ast.imports, wp);

        let mut file_env = execution_env::ExecutionEnv {
            variables_types: HashMap::new(),
            types_definitions: file.ast.types.clone(), // TODO: this clone looks bad :()
            funcs_definitions: file.ast.functions.clone(), // TODO: this clone looks bad too
            scope: None,
        };
        for import in &file.ast.imports {
            let import_path = import.module_path.alias();
            let imported_file = wp.files.get(&import_path).unwrap();

            for func_name in &import.functions {
                let x = imported_file.ast.functions.get(func_name).unwrap();
                file_env.funcs_definitions.insert(x.name.clone(), x.clone());
            }

            for typename in &import.typenames {
                let x = imported_file.ast.types.get(typename).unwrap();
                file_env.types_definitions.insert(x.name.clone(), x.clone());
            }
        }

        for typedef in file.ast.types.values() {
            file_env.scope = Some(typedef.clone());
            for method in typedef.methods.values() {
                statements::check_statements(&method.statements, &file_env);
            }
        }

        for funcdef in file.ast.functions.values() {
            file_env.scope = None;
            statements::check_statements(&funcdef.statements, &file_env);
        }
    }
}
