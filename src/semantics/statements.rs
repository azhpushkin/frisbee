use crate::ast::*;
use crate::semantic_checker::semantic_error::SemanticResult;

use super::expressions::ExprTypeChecker;
use super::semantic_error::sem_err;
use super::symbols::GlobalSymbolsInfo;

pub fn annotate_function_statements(
    func_decl: &mut FunctionDecl,
    file_module: &ModulePathAlias,
    scope: Option<String>,
    info: &GlobalSymbolsInfo,
) -> SemanticResult<()> {
    let mut expr_checker = ExprTypeChecker::new(info, file_module.clone(), scope);
    for arg in func_decl.args.iter() {
        expr_checker.add_variable(arg.name.clone(), arg.typename.clone())?;
    }

    for stmt in func_decl.statements.iter_mut() {
        match stmt {
            Statement::Expr(expr) => {
                expr_checker.calculate_and_annotate(expr)?;
            }
            Statement::Return(expr) => {
                let rettype = expr_checker.calculate_and_annotate(expr)?;
                if func_decl.rettype != rettype {
                    return sem_err!(
                        "Type mismatch in return! Expected {:?}, got {:?}",
                        func_decl,
                        rettype
                    );
                }
            }
            Statement::VarDecl(typename, varname) => {
                expr_checker.add_variable(varname.clone(), typename.clone())?;
            }
            Statement::VarDeclWithAssign(typename, varname, expr) => {
                let expr_type = expr_checker.calculate_and_annotate(expr)?;
                // TODO: [maybe]
                if !expr_type.eq(typename) {
                    return sem_err!(
                        "Type mismatch in assignment of {}! Expected {:?}, got {:?}",
                        varname,
                        typename,
                        expr_type
                    );
                }
                expr_checker.add_variable(varname.clone(), typename.clone())?;
            }
            _ => todo!(),
        }
    }
    Ok(())
}
