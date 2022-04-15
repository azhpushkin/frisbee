use crate::ast::Type;
use crate::semantics::aggregate::{ProgramAggregate, RawFunction};
use crate::semantics::light_ast::LStatement;
use crate::vm::opcodes::op;

use super::generator::{BytecodeGenerator, FunctionBytecode};
use super::globals::Globals;

pub fn generate_function_bytecode(
    func: &RawFunction,
    aggregate: &ProgramAggregate,
    globals: &mut Globals,
) -> Result<FunctionBytecode, String> {
    let arg_vars = func.args.iter().map(|arg| arg.0);

    let mut generator = BytecodeGenerator::new(
        aggregate,
        globals,
        arg_vars.enumerate().map(|(i, var)| (var, i as u8)).collect(),
    );

    for statement in func.body.iter() {
        match statement {
            LStatement::Expression(expr) => {
                generator.push_expr(expr);
                generator.push(op::POP);
            }
            LStatement::DeclareVar { var_type, name } => {
                // TODO: this should reserve the space for the variable
                generator.add_local(name);
                generator.push(op::RESERVE_ONE);
            }
            LStatement::AssignVar { name, value } => {
                generator.push_expr(value);
                generator.push_set_var(name);
            }
            LStatement::DeclareAndAssignVar {var_type, name, value} => {
                generator.add_local(name);
                generator.push_expr(value);
            }
            LStatement::Return(expr) => {
                generator.push_expr(expr);
                generator.push(op::RETURN);
            }
            _ => todo!(),
        }
    }

    Ok(generator.get_bytecode())
}
