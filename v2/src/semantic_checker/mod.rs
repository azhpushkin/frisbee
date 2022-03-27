use crate::loader::WholeProgram;

mod expressions;
mod module_types;
mod operators;
mod statements;
mod tests;

pub fn perform_checks(wp: &WholeProgram) {
    for (_, file) in &wp.files {
        module_types::check_collision_of_imports_and_definitions_per_module(&file.ast);
        module_types::check_type_is_not_referring_self(&file.ast);
        statements::check_statements(&file.ast)
    }
}
