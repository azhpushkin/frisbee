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
        generate_statement_bytecode(statement, &mut generator, None);
    }

    Ok(generator.get_bytecode())
}

fn generate_statement_bytecode<'a, 'b>(
    statement: &'a LStatement,
    generator: &mut BytecodeGenerator<'a, 'b>,
    loop_start: Option<usize>,    
) -> Vec<JumpPlaceholder> {
    let mut break_placeholders = vec![];
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
        LStatement::IfElse { condition, if_body: ifbody, else_body: elsebody } if elsebody.is_empty() => {
            generator.push_expr(condition);
            generator.push(op::JUMP_IF_FALSE);

            let placeholder_to_skip_ifbody: JumpPlaceholder = generator.push_placeholder();

            for statement in ifbody.iter() {
                let br = generate_statement_bytecode(statement, generator, loop_start);
                break_placeholders.extend(br);
            }
            let end_if_pos = generator.get_position() as u16;
            generator.fill_placeholder(&placeholder_to_skip_ifbody);
        }
        LStatement::IfElse { condition, if_body: ifbody, else_body: elsebody } => {
            generator.push_expr(condition);
            generator.push(op::JUMP_IF_FALSE);

            let placeholder_to_skip_ifbody = generator.push_placeholder();

            for statement in ifbody.iter() {
                let br = generate_statement_bytecode(statement, generator, loop_start);
                break_placeholders.extend(br);
            }
            generator.push(op::JUMP);
            let placeholder_to_skip_elsebody = generator.push_placeholder();
            generator.fill_placeholder(&placeholder_to_skip_ifbody);

            for statement in elsebody.iter() {
                let br = generate_statement_bytecode(statement, generator, loop_start);
                break_placeholders.extend(br);
            }
            generator.fill_placeholder(&placeholder_to_skip_elsebody);
        }
        LStatement::While { condition, body } => {
            let mut loop_breaks = vec![];
            let start_pos = generator.get_position();
            generator.push_expr(condition);
            
            generator.push(op::JUMP_IF_FALSE);
            let placeholder_to_skip_loop = generator.push_placeholder();

            for statement in body.iter() {
                let br = generate_statement_bytecode(statement, generator, Some(start_pos));
                loop_breaks.extend(br);
            }
            generator.push(op::JUMP_BACK);
            let placeholder_to_jump_back = generator.push_placeholder();

            // jump back to condition
            generator.fill_placeholder_backward(&placeholder_to_jump_back, start_pos);
            
            // after loop is done - fill jumps related to breaks and condition failure
            generator.fill_placeholder(&placeholder_to_skip_loop);            
            for break_placeholder in loop_breaks {
                generator.fill_placeholder(&break_placeholder);
            }
        },
        LStatement::Break => {
            generator.push(op::JUMP);
            break_placeholders.push(generator.push_placeholder());

        },
        LStatement::Continue => {
            generator.push(op::JUMP_BACK);
            let placeholder_to_jump_back = generator.push_placeholder();
            generator.fill_placeholder_backward(&placeholder_to_jump_back, loop_start.unwrap());
        },
    };
    break_placeholders
}
