use crate::ast::*;
use crate::semantic_checker::modules::annotate_type;
use crate::semantic_checker::semantic_error::SemanticResult;
use crate::semantic_checker::symbols::GlobalSymbolsInfo;

pub fn check_and_annotate_ast_in_place(
    file_ast: &mut FileAst,
    file_module: &ModulePathAlias,
    info: &GlobalSymbolsInfo,
) -> SemanticResult<()> {
    let types_per_file = &info.symbols_per_file[&file_module].typenames;
    for object_decl in file_ast.types.iter_mut() {
        for field in object_decl.fields.iter_mut() {
            (*field).typename = annotate_type(&field.typename, types_per_file)?;
        }

        for field in object_decl.methods.iter_mut() {
            (*field).rettype = annotate_type(&field.rettype, types_per_file)?;
            for param in field.args.iter_mut() {
                (*param).typename = annotate_type(&param.typename, types_per_file)?;
            }
        }
    }
    Ok(())
}
