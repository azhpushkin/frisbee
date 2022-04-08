use std::collections::HashMap;

use crate::ast::*;
use crate::vm::Op;

use super::expressions::ExprBytecodeGenerator;
use super::globals::Globals;

pub fn generate_function_bytecode(
    func: &FunctionDecl,
    globals: &mut Globals,
) -> Result<Vec<u8>, String> {
    let mut function_bytecode: Vec<u8> = vec![];

    let mut genexpr = ExprBytecodeGenerator::new(
        globals,
        func.args.iter().enumerate().map(|(i, arg)| (&arg.name, i as u8)).collect(),
    );

    for statement in func.statements.iter() {
        match statement {
            Statement::Expr(expr) => {
                genexpr.generate_and_flush(&expr, &mut function_bytecode)?;
                function_bytecode.push(Op::POP);
            }
            Statement::VarDecl(_, varname) => {
                genexpr.add_local(varname);
            }
            Statement::VarDeclWithAssign(_, varname, expr) => {
                genexpr.generate_and_flush(&expr, &mut function_bytecode)?;
                function_bytecode.push(Op::SET_VAR);
                function_bytecode.push(genexpr.add_local(varname));
            }
            Statement::Return(expr) => {
                genexpr.generate_and_flush(&expr, &mut function_bytecode)?;
                function_bytecode.push(Op::RETURN);
            }
            _ => todo!(),
        }
    }
    Ok(function_bytecode)
}
