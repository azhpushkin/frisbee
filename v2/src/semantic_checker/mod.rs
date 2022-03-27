use crate::loader::WholeProgram;

// TODO: define what kind of checks to do
mod module_types;
mod operators;
mod tests;

#[allow(dead_code)]
static TODO: &str = r#"


"#;

pub fn perform_checks(wp: &WholeProgram) {
    for (_, file) in &wp.files {
        module_types::check_collision_of_imports_and_definitions_per_module(&file.ast);
        module_types::check_type_is_not_referring_self(&file.ast);
    }
}
