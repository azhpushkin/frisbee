use std::collections::HashMap;

use crate::{
    ast::ModulePathAlias,
    loader::{LoadedFile, WholeProgram},
};

mod semantic_error;
// mod execution_env;
// mod expressions;
mod module_types;
// mod operators;
// mod statements;
// mod std_definitions;
mod tests;

pub fn perform_checks(wp: &WholeProgram) -> semantic_error::SemanticResult {
    let mut mappings_per_file: HashMap<ModulePathAlias, module_types::FileMappings> =
        HashMap::new();

    for (file_name, file) in wp.files.iter() {
        module_types::check_module_does_not_import_itself(file)?;

        let file_mappings = module_types::FileMappings {
            typenames: module_types::get_typenames_mapping(file)?,
            functions: module_types::get_functions_mapping(file)?,
        };
        mappings_per_file.insert(file_name.clone(), file_mappings);
    }

    Ok(())
}
