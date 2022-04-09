use crate::ast::*;
use crate::vm::Op;

use super::generator::BytecodeGenerator;
use super::globals::Globals;

pub fn generate_function_bytecode(
    func: &FunctionDecl,
    globals: &mut Globals,
) -> Result<Vec<u8>, String> {
    let arg_vars = func.args.iter().map(|arg| &arg.name);

    let mut generator = BytecodeGenerator::new(
        globals,
        arg_vars.enumerate().map(|(i, var)| (var, i as u8)).collect(),
    );

    for statement in func.statements.iter() {
        match statement {
            Statement::Expr(expr) => {
                generator.push_expr(expr);
                generator.push(Op::POP);
            }
            Statement::VarDecl(_, varname) => {
                // TODO: this should reserve the space for the variable
                generator.push(Op::LOAD_INT);
                generator.push(0);
                generator.add_local(varname);
            }
            Statement::VarDeclWithAssign(_, varname, expr) => {
                generator.push_expr(expr);
                generator.add_local(varname);
            }
            Statement::Return(expr) => {
                generator.push_expr(expr);
                generator.push(Op::RETURN);
            }
            _ => todo!(),
        }
    }

    Ok(generator.get_bytecode())
}
