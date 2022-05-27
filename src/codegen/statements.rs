use crate::ast::verified::{RawFunction, VStatement};
use crate::types::Type;
use crate::vm::opcodes::op;

use super::constants::ConstantsTable;
use super::generator::{BytecodeGenerator, FunctionBytecode, JumpPlaceholder};
use super::metadata::{CustomTypesMetadataTable, ListKindsMetadataTable};
use super::utils::{get_tuple_offset, get_type_size, unwrap_type_as};

pub fn generate_function_bytecode(
    func: &RawFunction,
    custom_types_meta: &CustomTypesMetadataTable,
    list_kinds_meta: &mut ListKindsMetadataTable,
    constants: &mut ConstantsTable,
) -> Result<FunctionBytecode, String> {
    let mut generator = BytecodeGenerator::new(custom_types_meta, list_kinds_meta, constants, func);

    for (local_name, local_type) in func.locals.iter() {
        generator.add_local(local_name, local_type);
    }
    let total_size: u8 = func.locals.iter().map(|p| get_type_size(&p.1)).sum();
    generator.push(op::RESERVE);
    generator.push(total_size);

    for statement in func.body.iter() {
        generator.push_statement(statement, None);
    }

    Ok(generator.get_bytecode())
}

impl<'a> BytecodeGenerator<'a> {
    pub fn push_statement(
        &mut self,
        statement: &'a VStatement,
        loop_start: Option<usize>,
    ) -> Vec<JumpPlaceholder> {
        let mut outer_break_placeholders = vec![];
        match statement {
            VStatement::Expression(expr) => {
                self.push_expr(expr);
                self.push_pop(&expr.expr_type);
            }
            VStatement::AssignLocal { name, tuple_indexes, value } => {
                self.push_expr(value);
                self.push_set_local(name, tuple_indexes);
            }
            VStatement::AssignToField { object, field, tuple_indexes, value } => {
                let object_type = unwrap_type_as!(&object.expr_type, Type::Custom);
                let object_meta = self.custom_types_meta.get_meta(object_type);
                // Push value before object, as we need to first pop a pointer
                // to access the memory before writing value to it
                self.push_expr(value);
                self.push_expr(object);

                

                let field_offset = object_meta.field_offsets[field];
                let field_type = &object_meta.field_types[field];

                let tuple_offset = get_tuple_offset(field_type, tuple_indexes);

                self.push(op::SET_OBJ_FIELD);
                self.push(field_offset + tuple_offset);
                self.push_type_size(&value.expr_type);
            }
            VStatement::AssignToList { list, index, tuple_indexes, value } => {
                let list_item_type = unwrap_type_as!(&list.expr_type, Type::List,);

                self.push_expr(value);
                self.push_expr(index);
                self.push_expr(list);

                let tuple_offset = get_tuple_offset(list_item_type.as_ref(), tuple_indexes);

                self.push(op::SET_LIST_ITEM);
                self.push(tuple_offset);
                self.push_type_size(&value.expr_type);
            }
            VStatement::Return(expr) => {
                self.push_expr(expr);
                self.push_return();
            }
            VStatement::IfElse { condition, if_body, else_body } if else_body.is_empty() => {
                self.push_expr(condition);
                self.push(op::JUMP_IF_FALSE);

                let placeholder_to_skip_ifbody: JumpPlaceholder = self.push_placeholder();

                for statement in if_body.iter() {
                    let breaks = self.push_statement(statement, loop_start);
                    outer_break_placeholders.extend(breaks);
                }
                self.fill_placeholder(&placeholder_to_skip_ifbody);
            }
            VStatement::IfElse { condition, if_body, else_body } => {
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
            VStatement::While { condition, body } => {
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
            VStatement::Break => {
                self.push(op::JUMP);
                outer_break_placeholders.push(self.push_placeholder());
            }
            VStatement::Continue => {
                self.push(op::JUMP_BACK);
                let placeholder_to_jump_back = self.push_placeholder();
                self.fill_placeholder_backward(&placeholder_to_jump_back, loop_start.unwrap());
            }
            VStatement::AssignToCurrentActiveField { active_type, field, value, tuple_indexes } => {
                // Push value before object, as we need to first pop a pointer
                // to access the memory before writing value to it
                self.push_expr(value);

                let field_offset =
                    self.custom_types_meta.get_meta(active_type).field_offsets[field];
                let tuple_offset = get_tuple_offset(&value.expr_type, tuple_indexes);

                self.push(op::SET_CURRENT_ACTIVE_FIELD);
                self.push(field_offset + tuple_offset);
                self.push_type_size(&value.expr_type);
            }
            VStatement::SendMessage { active, receiver, args } => {
                self.push_expr(active);
                for arg in args.iter() {
                    self.push_expr(arg);
                }
                self.push(op::SEND_MESSAGE);
                self.push_function_placeholder(receiver);
            }
        };
        outer_break_placeholders
    }
}
