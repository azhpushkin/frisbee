use std::collections::HashMap;
use std::iter::Extend;
use std::iter::Iterator;

use semantic_error::{sem_err, SemanticResult};
use symbols::*;

use crate::ast::ModulePathAlias;
use crate::loader::WholeProgram;

mod expressions;
mod modules;
mod operators;
mod semantic_error;
mod statements;
mod annotations;
mod std_definitions;
mod symbols;
mod tests;

pub fn check_and_annotate_symbols(wp: &mut WholeProgram) -> SemanticResult<GlobalSymbolsInfo> {
    let mut symbols_per_file: HashMap<ModulePathAlias, SymbolOriginsPerFile> = HashMap::new();
    let mut global_signatures =
        GlobalSignatures { typenames: HashMap::new(), functions: HashMap::new() };

    for (file_name, file) in wp.files.iter_mut() {
        modules::check_module_does_not_import_itself(file)?;

        let file_mappings = SymbolOriginsPerFile {
            typenames: modules::get_typenames_origins(file)?,
            functions: modules::get_functions_origins(file)?,
        };

        for class_decl in file.ast.types.iter_mut() {
            modules::check_class_does_not_contains_itself(class_decl)?;
            modules::check_class_has_no_duplicated_methods(class_decl)?;

            annotations::annotate_class_decl(class_decl, &file_mappings.typenames)?;
        }
        for func_decl in file.ast.functions.iter_mut() {
            annotations::annotate_function_decl(func_decl, &file_mappings.typenames)?;
        }

        global_signatures.typenames.extend(modules::get_typenames_signatures(file));
        global_signatures.functions.extend(modules::get_functions_signatures(file));
        symbols_per_file.insert(file_name.clone(), file_mappings);
    }

    let any_missing_function_origin = symbols_per_file
        .values()
        .flat_map(|s| s.functions.values())
        .find(|f| !global_signatures.functions.contains_key(f));
    let any_missing_type_origin = symbols_per_file
        .values()
        .flat_map(|s| s.typenames.values())
        .find(|t| !global_signatures.typenames.contains_key(t));

    if any_missing_function_origin.is_some() {
        return sem_err!(
            "Function {:?} is not defined!",
            any_missing_function_origin.unwrap()
        );
    }
    if any_missing_type_origin.is_some() {
        return sem_err!(
            "Type {:?} is not defined!",
            any_missing_type_origin.unwrap()
        );
    }

    Ok(GlobalSymbolsInfo { symbols_per_file, global_signatures })
}

pub fn check_and_annotate_statements(
    wp: &mut WholeProgram,
    symbols_info: &GlobalSymbolsInfo,
) -> SemanticResult<()> {
    for (file_name, file) in wp.files.iter_mut() {
        annotations::check_and_annotate_ast_in_place(&mut file.ast, file_name, symbols_info)?;
    }

    Ok(())
}
