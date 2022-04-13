use crate::semantics::aggregate::{RawFunction, ProgramAggregate};
use crate::semantics::light_ast::LStatement;
use crate::vm::op;
use crate::ast::Type;

use super::generator::BytecodeGenerator;
use super::globals::Globals;



pub fn generate_function_bytecode(
    func: &RawFunction,
    aggregate: &ProgramAggregate,
    globals: &mut Globals,
) -> Result<Vec<u8>, String> {
    let arg_vars = func.args.iter().map(|arg| &arg.name);

    let mut generator = BytecodeGenerator::new(
        aggregate,
        &mut globals,
        arg_vars.enumerate().map(|(i, var)| (var, i as u8)).collect(),
    );

    for statement in func.body.iter() {
        match statement {
            LStatement::Expr(expr) => {
                generator.push_expr(expr);
                generator.push(op::POP);
            }
            LStatement::DeclareVar{var_type, name} => {
                // TODO: this should reserve the space for the variable
                generator.push(op::LOAD_INT);
                generator.push(0);
                generator.add_local(name);
            }
            LStatement::AssignVar{name, value} => {
                generator.push_expr(expr);
                generator.add_local(varname);
            }
            Statement::Return(expr) => {
                generator.push_expr(expr);
                generator.push(op::RETURN);
            }
            _ => todo!(),
        }
    }

    Ok(generator.get_bytecode())
}
