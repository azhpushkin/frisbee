use std::collections::HashMap;
use std::iter::Extend;
use std::iter::Iterator;

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

use semantic_error::{sem_err, SemanticResult};

pub fn perform_checks(wp: &WholeProgram) -> SemanticResult<()> {
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
        global_types_mapping.extend(modules::get_typenames_signatures(
            file,
            &file_mappings.typenames,
        )?);
        global_funcs_mapping.extend(modules::get_functions_signatures(
            file,
            &file_mappings.typenames,
        )?);
        mappings_per_file.insert(file_name.clone(), file_mappings);
    }

    for (file_name, file) in wp.files.iter() {
        let file_mappings = mappings_per_file
            .get(file_name)
            .expect("Mappings not found!");
        let unknown_type = file_mappings
            .typenames
            .values()
            .find(|t| !global_types_mapping.contains_key(t));
        let unknown_func = file_mappings
            .functions
            .values()
            .find(|f| !global_funcs_mapping.contains_key(f));

        // Check that all
        if let Some(t) = unknown_type {
            return sem_err!("{:?} type in module {:?} is non-existing", t, file_name);
        }
        if let Some(f) = unknown_func {
            return sem_err!("{:?} function in module {:?} is non-existing", f, file_name);
        }
    }

    Ok(())
}
