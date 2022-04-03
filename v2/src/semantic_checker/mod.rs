use std::collections::HashMap;
use std::iter::Extend;
use std::iter::Iterator;

use semantic_error::{sem_err, SemanticResult};
use symbols::*;

use crate::ast::ModulePathAlias;
use crate::loader::{LoadedFile, WholeProgram};

// mod expressions;
mod modules;
mod operators;
mod semantic_error;
// mod statements;
mod annotations;
mod std_definitions;
mod symbols;
mod tests;

pub fn check_and_gather_symbols_info(wp: &WholeProgram) -> SemanticResult<GlobalSymbolsInfo> {
    let mut symbols_per_file: HashMap<ModulePathAlias, SymbolOriginsPerFile> = HashMap::new();
    let mut global_signatures =
        GlobalSignatures { typenames: HashMap::new(), functions: HashMap::new() };

    for (file_name, file) in wp.files.iter() {
        modules::check_module_does_not_import_itself(file)?;

        let file_mappings = SymbolOriginsPerFile {
            typenames: modules::get_typenames_mapping(file)?,
            functions: modules::get_functions_mapping(file)?,
        };
        let file_type_signatures =
            modules::get_typenames_signatures(file, &file_mappings.typenames)?;

        let file_function_signatures =
            modules::get_functions_signatures(file, &file_mappings.typenames)?;

        global_signatures.typenames.extend(file_type_signatures);
        global_signatures.functions.extend(file_function_signatures);
        symbols_per_file.insert(file_name.clone(), file_mappings);
    }

    for (file_name, file) in wp.files.iter() {
        let file_mappings = symbols_per_file
            .get(file_name)
            .expect("Mappings not found!");
        let unknown_type = file_mappings
            .typenames
            .values()
            .find(|t| !global_signatures.typenames.contains_key(t));
        let unknown_func = file_mappings
            .functions
            .values()
            .find(|f| !global_signatures.functions.contains_key(f));

        // Check that all
        if let Some(t) = unknown_type {
            return sem_err!("{:?} type in module {:?} is non-existing", t, file_name);
        }
        if let Some(f) = unknown_func {
            return sem_err!("{:?} function in module {:?} is non-existing", f, file_name);
        }
    }

    Ok(GlobalSymbolsInfo { symbols_per_file, global_signatures })
}

pub fn annotate_whole_program(
    wp: &mut WholeProgram,
    symbols_info: &GlobalSymbolsInfo,
) -> SemanticResult<()> {
    for (file_name, file) in wp.files.iter_mut() {
        annotations::check_and_annotate_ast_in_place(&mut file.ast, file_name, symbols_info)?;
    }

    Ok(())
}
