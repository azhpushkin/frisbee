use std::collections::HashMap;

use crate::ast::verified::RawFunction;
use crate::symbols::SymbolFunc;
use crate::types::VerifiedType;
use crate::vm::opcodes::op;

use super::constants::{Constant, ConstantsTable};
use super::metadata::{ListMetadataTable, TypesMetadataTable};
use super::utils::{get_tuple_offset, get_tuple_subitem_size, get_type_size};

pub type CallPlaceholders = (usize, SymbolFunc);

pub struct FunctionBytecode {
    pub name: SymbolFunc,
    pub bytecode: Vec<u8>,
    pub call_placeholders: Vec<CallPlaceholders>,
    pub locals_size: usize,
    pub pointer_mapping: Vec<usize>,
}
pub struct JumpPlaceholder {
    position: usize,
}

pub struct BytecodeGenerator<'a> {
    pub types_meta: &'a TypesMetadataTable,
    pub list_types_meta: &'a mut ListMetadataTable,
    constants: &'a mut ConstantsTable,
    locals: HashMap<&'a str, u8>,
    locals_offset: u8,
    locals_types: HashMap<&'a str, &'a VerifiedType>,
    locals_order: Vec<&'a str>,
    return_type: &'a VerifiedType,
    bytecode: FunctionBytecode,
}

impl<'a> BytecodeGenerator<'a> {
    pub fn new(
        types_meta: &'a TypesMetadataTable,
        list_types_meta: &'a mut ListMetadataTable,
        constants: &'a mut ConstantsTable,
        function: &'a RawFunction,
    ) -> Self {
        let mut locals: HashMap<&'a str, u8> = HashMap::new();
        let mut locals_offset: u8 = get_type_size(&function.return_type);
        let mut locals_types = HashMap::new();
        let mut locals_order = vec![];

        for (local_name, local_type) in function.args.iter() {
            locals.insert(local_name, locals_offset);
            locals_offset += get_type_size(local_type);
            locals_types.insert(local_name.as_str(), local_type);
            locals_order.push(local_name.as_str());
        }

        BytecodeGenerator {
            types_meta,
            list_types_meta,
            constants,
            locals,
            locals_offset,
            locals_types,
            locals_order,
            return_type: &function.return_type,
            bytecode: FunctionBytecode {
                name: function.name.clone(),
                bytecode: vec![],
                call_placeholders: vec![],
                locals_size: locals_offset as usize,
                pointer_mapping: vec![],
            },
        }
    }

    pub fn add_local(&mut self, varname: &'a str, t: &'a VerifiedType) {
        self.locals.insert(varname, self.locals_offset);
        self.locals_types.insert(varname, t);
        self.locals_offset += get_type_size(t);
        self.locals_order.push(varname);
    }

    pub fn push_get_local(&mut self, varname: &str) {
        let var_pos = *self.locals.get(varname).unwrap();
        self.push(op::GET_LOCAL);
        self.push(var_pos);
        self.push_type_size(self.locals_types[varname]);
    }

    pub fn push_set_local(&mut self, varname: &str, tuple_indexes: &[usize]) {
        let var_pos = *self.locals.get(varname).unwrap();
        let offset = get_tuple_offset(self.locals_types[varname], tuple_indexes);
        self.push(op::SET_LOCAL);
        self.push(var_pos + offset);
        self.push(get_tuple_subitem_size(
            self.locals_types[varname],
            tuple_indexes,
        ));
    }

    pub fn push_set_return(&mut self) {
        self.push(op::SET_LOCAL);
        self.push(0);
        self.push_type_size(self.return_type);
    }

    pub fn push_constant(&mut self, constant: Constant) {
        let constant_pos = self.constants.get_constant(constant);
        self.push(constant_pos);
    }

    pub fn push(&mut self, opcode: u8) {
        self.bytecode.bytecode.push(opcode);
    }

    pub fn push_type_size(&mut self, t: &VerifiedType) {
        self.push(get_type_size(t))
    }

    pub fn push_reserve(&mut self, for_type: &VerifiedType) {
        let reserve_size = get_type_size(for_type);
        if reserve_size > 0 {
            self.push(op::RESERVE);
            self.push_type_size(for_type);
        }
    }

    pub fn push_pop(&mut self, for_type: &VerifiedType) {
        let pop_size = get_type_size(for_type);
        if pop_size > 0 {
            self.push(op::POP);
            self.push_type_size(for_type);
        }
    }

    pub fn push_function_placeholder(&mut self, func: &SymbolFunc) {
        self.bytecode
            .call_placeholders
            .push((self.bytecode.bytecode.len(), func.clone()));
        self.push(0);
        self.push(0);
    }

    pub fn push_placeholder(&mut self) -> JumpPlaceholder {
        let placeholder_pos = self.get_position();
        self.bytecode.bytecode.resize(placeholder_pos + 2, 0);
        JumpPlaceholder { position: placeholder_pos }
    }

    pub fn get_position(&self) -> usize {
        self.bytecode.bytecode.len()
    }

    pub fn fill_placeholder(&mut self, placeholder: &JumpPlaceholder) {
        // -2 as this is length of placeholder in the program
        let diff = ((self.get_position() - placeholder.position - 2) as u16).to_be_bytes();
        self.bytecode.bytecode[placeholder.position] = diff[0];
        self.bytecode.bytecode[placeholder.position + 1] = diff[1];
    }

    pub fn fill_placeholder_backward(&mut self, placeholder: &JumpPlaceholder, jump_to: usize) {
        // placeholder.position is more than jump_to
        let diff = ((placeholder.position - jump_to + 2) as u16).to_be_bytes();
        self.bytecode.bytecode[placeholder.position] = diff[0];
        self.bytecode.bytecode[placeholder.position + 1] = diff[1];
    }

    pub fn get_bytecode(self) -> FunctionBytecode {
        self.bytecode
    }
}
