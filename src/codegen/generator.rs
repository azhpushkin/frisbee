use std::collections::HashMap;

use crate::ast::verified::RawFunction;
use crate::runtime::opcodes::op;
use crate::symbols::SymbolFunc;
use crate::types::VerifiedType;

use super::constants::{Constant, ConstantsTable};
use super::metadata::{CustomTypesMetadataTable, ListKindsMetadataTable};
use super::utils::get_type_size;

pub type CallPlaceholders = (usize, SymbolFunc);

pub struct FunctionBytecode {
    pub name: SymbolFunc,
    pub bytecode: Vec<u8>,
    pub call_placeholders: Vec<CallPlaceholders>,
    pub locals_size: usize,
    pub args_size: usize,
    pub args_pointer_mapping: Vec<usize>,
}
pub struct JumpPlaceholder {
    position: usize,
}

pub struct BytecodeGenerator<'a> {
    pub custom_types_meta: &'a CustomTypesMetadataTable,
    pub list_kinds_meta: &'a mut ListKindsMetadataTable,
    pub constants: &'a mut ConstantsTable,
    pub locals: HashMap<&'a str, u8>,
    pub locals_offset: u8,
    pub locals_types: HashMap<&'a str, &'a VerifiedType>,
    pub locals_order: Vec<&'a str>,
    pub return_type: &'a VerifiedType,
    bytecode: FunctionBytecode,
}

impl<'a> BytecodeGenerator<'a> {
    pub fn new(
        custom_types_meta: &'a CustomTypesMetadataTable,
        list_kinds_meta: &'a mut ListKindsMetadataTable,
        constants: &'a mut ConstantsTable,
        function: &'a RawFunction,
    ) -> Self {
        let mut locals: HashMap<&'a str, u8> = HashMap::new();
        let mut locals_offset: u8 = 0;
        let mut locals_types = HashMap::new();
        let mut locals_order = vec![];

        for (local_name, local_type) in function.args.iter() {
            locals.insert(local_name, locals_offset);
            locals_offset += get_type_size(local_type);
            locals_types.insert(local_name.as_str(), local_type);
            locals_order.push(local_name.as_str());
        }

        BytecodeGenerator {
            custom_types_meta,
            list_kinds_meta,
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
                args_size: function.args.types.iter().map(get_type_size).sum::<u8>() as usize,
                args_pointer_mapping: vec![],
            },
        }
    }

    pub fn add_local(&mut self, varname: &'a str, t: &'a VerifiedType) {
        self.locals.insert(varname, self.locals_offset);
        self.locals_types.insert(varname, t);
        self.locals_offset += get_type_size(t);
        self.locals_order.push(varname);
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
