use crate::semantics::aggregate::RawFunction;
use crate::semantics::light_ast::LStatement;
use crate::vm::opcodes::op;

use super::constants::ConstantsTable;
use super::generator::{BytecodeGenerator, FunctionBytecode, JumpPlaceholder};
use super::types_metadata::TypeMetadataTable;

pub fn generate_function_bytecode(
    func: &RawFunction,
    types_meta: &TypeMetadataTable,
    constants: &mut ConstantsTable,
) -> Result<FunctionBytecode, String> {
    let mut generator = BytecodeGenerator::new(
        types_meta,
        constants,
        func.args.iter().collect(),
        &func.return_type,
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
            generator.push_pop(&expr.expr_type);
        }
        LStatement::DeclareVar { var_type, name } => {
            generator.add_local(name, var_type);
            generator.push_reserve(var_type);
        }
        LStatement::AssignLocal { name, tuple_indexes, value } => {
            generator.push_expr(value);
            generator.push_set_local(name, tuple_indexes);
        }
        LStatement::DeclareAndAssignVar { var_type, name, value } => {
            generator.add_local(name, var_type);
            generator.push_expr(value);
        }
        LStatement::AssignToPointer { left, right } => todo!(),
        LStatement::Return(expr) => {
            generator.push_expr(expr);
            generator.push_set_return();
            generator.push(op::RETURN);
        }
        LStatement::IfElse { condition, if_body, else_body } if else_body.is_empty() => {
            generator.push_expr(condition);
            generator.push(op::JUMP_IF_FALSE);

            let placeholder_to_skip_ifbody: JumpPlaceholder = generator.push_placeholder();

            for statement in if_body.iter() {
                let br = generate_statement_bytecode(statement, generator, loop_start);
                break_placeholders.extend(br);
            }
            generator.fill_placeholder(&placeholder_to_skip_ifbody);
        }
        LStatement::IfElse { condition, if_body, else_body } => {
            generator.push_expr(condition);
            generator.push(op::JUMP_IF_FALSE);

            let placeholder_to_skip_ifbody = generator.push_placeholder();

            for statement in if_body.iter() {
                let br = generate_statement_bytecode(statement, generator, loop_start);
                break_placeholders.extend(br);
            }
            generator.push(op::JUMP);
            let placeholder_to_skip_elsebody = generator.push_placeholder();
            generator.fill_placeholder(&placeholder_to_skip_ifbody);

            for statement in else_body.iter() {
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
        }
        LStatement::Break => {
            generator.push(op::JUMP);
            break_placeholders.push(generator.push_placeholder());
        }
        LStatement::Continue => {
            generator.push(op::JUMP_BACK);
            let placeholder_to_jump_back = generator.push_placeholder();
            generator.fill_placeholder_backward(&placeholder_to_jump_back, loop_start.unwrap());
        }
    };
    break_placeholders
}
