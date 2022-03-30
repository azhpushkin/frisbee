use crate::loader::WholeProgram;

mod semantic_error;
// mod execution_env;
// mod expressions;
// mod module_types;
// mod operators;
// mod statements;
// mod std_definitions;
// mod tests;

// pub fn perform_checks(wp: &WholeProgram) {
//     for (file_name, file) in &wp.files {
//         module_types::check_collision_of_imports_and_definitions_per_module(&file.ast);
//         module_types::check_type_is_not_referring_self(&file.ast);
//         module_types::check_imports_of_itself(file);
//         module_types::check_imports_are_correct(&file.ast.imports, wp);
//     }
// }
