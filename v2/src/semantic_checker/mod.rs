use std::collections::HashMap;

use crate::{
    ast::ModulePathAlias,
    loader::{LoadedFile, WholeProgram},
};

mod semantic_error;
// mod execution_env;
// mod expressions;
mod modules;
// mod operators;
// mod statements;
// mod std_definitions;
mod tests;

pub fn perform_checks(wp: &WholeProgram) -> semantic_error::SemanticResult<()> {
    let mut mappings_per_file: HashMap<ModulePathAlias, modules::FileMappings> = HashMap::new();
    let mut global_types_mapping: HashMap<modules::ObjectPath, modules::ObjectSignature> =
        HashMap::new();
    let mut global_funcs_mapping: HashMap<modules::ObjectPath, modules::FunctionSignature> =
        HashMap::new();

    for (file_name, file) in wp.files.iter() {
        modules::check_module_does_not_import_itself(file)?;

        let file_mappings = modules::FileMappings {
            typenames: modules::get_typenames_mapping(file)?,
            functions: modules::get_functions_mapping(file)?,
        };
        modules::get_typenames_signatures(file, &file_mappings.typenames)?;
        modules::get_functions_signatures(file, &file_mappings.typenames)?;

        mappings_per_file.insert(file_name.clone(), file_mappings);
    }

    Ok(())
}
