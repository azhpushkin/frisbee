use crate::semantics::aggregate::RawFunction;
use crate::semantics::light_ast::{LExpr, LStatement};
use crate::semantics::symbols::SymbolType;
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
        generator.push_statement(statement, None);
    }

    Ok(generator.get_bytecode())
}

impl<'a, 'b> BytecodeGenerator<'a, 'b> {
    pub fn push_statement(
        &mut self,
        statement: &'a LStatement,
        loop_start: Option<usize>,
    ) -> Vec<JumpPlaceholder> {
        let mut outer_break_placeholders = vec![];
        match statement {
            LStatement::Expression(expr) => {
                self.push_expr(expr);
                self.push_pop(&expr.expr_type);
            }
            LStatement::DeclareVar { var_type, name } => {
                self.add_local(name, var_type);
                self.push_reserve(var_type);
            }
            LStatement::AssignLocal { name, tuple_indexes, value } => {
                self.push_expr(value);
                self.push_set_local(name, tuple_indexes);
            }
            LStatement::DeclareAndAssignVar { var_type, name, value } => {
                self.add_local(name, var_type);
                self.push_expr(value);
            }
            LStatement::AssignToField { object, field, value } => {
                let object_type: SymbolType = object.expr_type.clone().into();
                // Push value before object, as we need to first pop a pointer
                // to access the memory before writing value to it
                self.push_expr(value);
                self.push_expr(&object);
                
                self.push(op::SET_TO_HEAP);
                self.push(self.types_meta.get(&object_type).field_offsets[field]);
                self.push(self.types_meta.get(&object_type).field_sizes[field]);
            }
            LStatement::Return(expr) => {
                self.push_expr(expr);
                self.push_set_return();
                self.push(op::RETURN);
            }
            LStatement::IfElse { condition, if_body, else_body } if else_body.is_empty() => {
                self.push_expr(condition);
                self.push(op::JUMP_IF_FALSE);

                let placeholder_to_skip_ifbody: JumpPlaceholder = self.push_placeholder();

                for statement in if_body.iter() {
                    let breaks = self.push_statement(statement, loop_start);
                    outer_break_placeholders.extend(breaks);
                }
                self.fill_placeholder(&placeholder_to_skip_ifbody);
            }
            LStatement::IfElse { condition, if_body, else_body } => {
                self.push_expr(condition);
                self.push(op::JUMP_IF_FALSE);

                let placeholder_to_skip_ifbody = self.push_placeholder();

                for statement in if_body.iter() {
                    let breaks = self.push_statement(statement, loop_start);
                    outer_break_placeholders.extend(breaks);
                }
                self.push(op::JUMP);
                let placeholder_to_skip_elsebody = self.push_placeholder();
                self.fill_placeholder(&placeholder_to_skip_ifbody);

                for statement in else_body.iter() {
                    let breaks = self.push_statement(statement, loop_start);
                    outer_break_placeholders.extend(breaks);
                }
                self.fill_placeholder(&placeholder_to_skip_elsebody);
            }
            LStatement::While { condition, body } => {
                let mut loop_breaks = vec![];
                let start_pos = self.get_position();
                self.push_expr(condition);

                self.push(op::JUMP_IF_FALSE);
                let placeholder_to_skip_loop = self.push_placeholder();

                for statement in body.iter() {
                    let breaks = self.push_statement(statement, Some(start_pos));
                    loop_breaks.extend(breaks);
                }
                self.push(op::JUMP_BACK);
                let placeholder_to_jump_back = self.push_placeholder();

                // jump back to condition
                self.fill_placeholder_backward(&placeholder_to_jump_back, start_pos);

                // after loop is done - fill jumps related to breaks and condition failure
                self.fill_placeholder(&placeholder_to_skip_loop);
                for break_placeholder in loop_breaks {
                    self.fill_placeholder(&break_placeholder);
                }
            }
            LStatement::Break => {
                self.push(op::JUMP);
                outer_break_placeholders.push(self.push_placeholder());
            }
            LStatement::Continue => {
                self.push(op::JUMP_BACK);
                let placeholder_to_jump_back = self.push_placeholder();
                self.fill_placeholder_backward(&placeholder_to_jump_back, loop_start.unwrap());
            }
        };
        outer_break_placeholders
    }
}
