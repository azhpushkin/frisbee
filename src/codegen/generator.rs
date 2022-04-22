use std::collections::HashMap;

use crate::semantics::aggregate::ProgramAggregate;
use crate::semantics::symbols::SymbolFunc;
use crate::types::Type;
use crate::vm::opcodes::op;

use super::constants::Constant;
use super::globals::Globals;

pub type CallPlaceholders = (usize, SymbolFunc);

pub struct FunctionBytecode {
    pub bytecode: Vec<u8>,
    pub call_placeholders: Vec<CallPlaceholders>,
}
pub struct JumpPlaceholder {
    position: usize,
}

fn get_type_size(t: &Type) -> u8 {
    match t {
        Type::Int => 1,
        Type::Float => 1,
        Type::Bool => 1,
        Type::String => 1,
        Type::Maybe(inner) => get_type_size(inner.as_ref()) + 1,
        Type::Tuple(items) => items.iter().map(|t| get_type_size(t)).sum(),
        Type::List(_) => 1,
        Type::Ident(_) => 1,
    }
}

pub struct BytecodeGenerator<'a, 'b> {
    aggregate: &'a ProgramAggregate,
    globals: &'b mut Globals,
    locals: HashMap<&'a String, u8>,
    locals_offset: u8,
    locals_types: HashMap<&'a String, Type>,
    bytecode: FunctionBytecode,
}

impl<'a, 'b> BytecodeGenerator<'a, 'b> {
    pub fn new(
        aggregate: &'a ProgramAggregate,
        globals: &'b mut Globals,
        initial_locals: Vec<(&'a String, &'a Type)>,
        return_type: &'a Type,
    ) -> Self {
        let mut locals: HashMap<&'a String, u8> = HashMap::new();
        let mut locals_offset: u8 = get_type_size(return_type);
        let mut locals_types = HashMap::new();
        
        for (local_name, local_type) in initial_locals {
            locals.insert(local_name, locals_offset);
            locals_offset += get_type_size(local_type);
            locals_types.insert(local_name, local_type.clone());
        }

        BytecodeGenerator {
            aggregate,
            globals,
            locals,
            locals_offset,
            locals_types,
            bytecode: FunctionBytecode { bytecode: vec![], call_placeholders: vec![] },
        }
    }

    pub fn add_local(&mut self, varname: &'a String, t: &Type) {
        self.locals.insert(varname, self.locals_offset);
        self.locals_types.insert(varname, t.clone());
        self.locals_offset += get_type_size(t);
    }

    pub fn push_get_var(&mut self, varname: &String) {
        let var_pos = self.locals.get(varname).unwrap().clone();
        self.push(op::GET_VAR);
        self.push(var_pos + 1);
        self.push(get_type_size(&self.locals_types[varname]));
    }

    pub fn push_set_var(&mut self, varname: &String) {
        let var_pos = self.locals.get(varname).unwrap().clone();
        self.push(op::SET_VAR);
        self.push(var_pos + 1);
        self.push(get_type_size(&self.locals_types[varname]));
    }

    pub fn push_set_return(&mut self) {
        self.push(op::SET_VAR);
        self.push(0);
    }

    pub fn push_constant(&mut self, constant: Constant) {
        let constant_pos = self.globals.get_constant(constant);
        self.push(constant_pos);
    }

    pub fn push(&mut self, opcode: u8) {
        self.bytecode.bytecode.push(opcode);
    }

    pub fn push_reserve(&mut self, for_type: &Type) {
        self.push(op::RESERVE);
        self.push(get_type_size(for_type) as u8);
    }

    pub fn push_pop(&mut self, for_type: &Type) {
        self.push(op::POP);
        self.push(get_type_size(for_type) as u8);
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

    pub fn get_bytecode(&mut self) -> FunctionBytecode {
        let mut temp = FunctionBytecode { bytecode: vec![], call_placeholders: vec![] };
        std::mem::swap(&mut self.bytecode, &mut temp);
        temp
    }
}
