use std::collections::HashMap;
use std::iter::Extend;
use std::iter::Iterator;

use crate::ast::ModulePathAlias;
use crate::loader::{LoadedFile, WholeProgram};

mod expressions;
mod modules;
mod operators;
mod semantic_error;
mod type_env;
// mod statements;
mod std_definitions;
mod tests;

use modules::*;
use semantic_error::{sem_err, SemanticResult};

pub fn perform_checks(wp: &WholeProgram) -> SemanticResult<()> {
    let mut mappings_per_file: HashMap<ModulePathAlias, SymbolOriginsPerFile> = HashMap::new();
    let mut global_mapping =
        GlobalSignatures { typenames: HashMap::new(), functions: HashMap::new() };

    for (file_name, file) in wp.files.iter() {
        check_module_does_not_import_itself(file)?;

        let file_mappings = SymbolOriginsPerFile {
            typenames: get_typenames_mapping(file)?,
            functions: get_functions_mapping(file)?,
        };
        let file_type_signatures = get_typenames_signatures(file, &file_mappings.typenames)?;

        let file_function_signatures = get_functions_signatures(file, &file_mappings.typenames)?;

        global_mapping.typenames.extend(file_type_signatures);
        global_mapping.functions.extend(file_function_signatures);
        mappings_per_file.insert(file_name.clone(), file_mappings);
    }

    for (file_name, file) in wp.files.iter() {
        let file_mappings = mappings_per_file
            .get(file_name)
            .expect("Mappings not found!");
        let unknown_type = file_mappings
            .typenames
            .values()
            .find(|t| !global_mapping.typenames.contains_key(t));
        let unknown_func = file_mappings
            .functions
            .values()
            .find(|f| !global_mapping.functions.contains_key(f));

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
