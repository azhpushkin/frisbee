use crate::ast::Type;
use crate::semantics::aggregate::{ProgramAggregate, RawFunction};
use crate::semantics::light_ast::LStatement;
use crate::vm::opcodes::op;

use super::generator::{BytecodeGenerator, FunctionBytecode, JumpPlaceholder};
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
        generate_statement_bytecode(statement, &mut generator);
    }

    Ok(generator.get_bytecode())
}

fn generate_statement_bytecode<'a, 'b>(
    statement: &'a LStatement,
    generator: &mut BytecodeGenerator<'a, 'b>,
) {
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
        LStatement::DeclareAndAssignVar { var_type, name, value } => {
            generator.add_local(name);
            generator.push_expr(value);
        }
        LStatement::Return(expr) => {
            generator.push_expr(expr);
            generator.push_set_return();
            generator.push(op::RETURN);
        }
        LStatement::IfElse { condition, ifbody, elsebody } if elsebody.is_empty() => {
            generator.push_expr(condition);
            generator.push(op::JUMP_IF_FALSE);

            let placeholder_to_skip_ifbody: JumpPlaceholder = generator.push_placeholder();

            for statement in ifbody.iter() {
                generate_statement_bytecode(statement, generator);
            }
            let end_if_pos = generator.get_position() as u16;
            generator.fill_placeholder(&placeholder_to_skip_ifbody, generator.get_position());
        }
        LStatement::IfElse { condition, ifbody, elsebody } => {
            generator.push_expr(condition);
            generator.push(op::JUMP_IF_FALSE);

            let placeholder_to_skip_ifbody = generator.push_placeholder();

            for statement in ifbody.iter() {
                generate_statement_bytecode(statement, generator);
            }
            generator.push(op::JUMP);
            let placeholder_to_skip_elsebody = generator.push_placeholder();
            generator.fill_placeholder(&placeholder_to_skip_ifbody, generator.get_position());

            for statement in elsebody.iter() {
                generate_statement_bytecode(statement, generator);
            }
            generator.fill_placeholder(&placeholder_to_skip_elsebody, generator.get_position());
        }
        _ => todo!(),
    }
}
