use std::collections::HashMap;

use crate::semantics::aggregate::ProgramAggregate;
use crate::semantics::symbols::SymbolFunc;
use crate::vm::opcodes::op;

use super::constants::Constant;
use super::globals::Globals;

pub type CallPlaceholders = (usize, SymbolFunc);

pub struct FunctionBytecode {
    pub bytecode: Vec<u8>,
    pub call_placeholders: Vec<CallPlaceholders>,
}

pub struct BytecodeGenerator<'a, 'b> {
    aggregate: &'a ProgramAggregate,
    globals: &'b mut Globals,
    locals: HashMap<&'a String, u8>,
    bytecode: FunctionBytecode,
}

impl<'a, 'b> BytecodeGenerator<'a, 'b> {
    pub fn new(
        aggregate: &'a ProgramAggregate,
        globals: &'b mut Globals,
        locals: HashMap<&'a String, u8>,
    ) -> Self {
        BytecodeGenerator {
            aggregate,
            globals,
            locals,
            bytecode: FunctionBytecode { bytecode: vec![], call_placeholders: vec![] },
        }
    }

    pub fn add_local(&mut self, varname: &'a String) {
        // TODO: add type info for offsets
        self.locals.insert(varname, self.locals.len() as u8);
        self.push(op::RESERVE_ONE);
    }

    pub fn push_get_var(&mut self, varname: &String) {
        let var_pos = self.locals.get(varname).unwrap().clone();
        self.push(op::GET_VAR);
        self.push(var_pos);
    }

    pub fn push_set_var(&mut self, varname: &String) {
        let var_pos = self.locals.get(varname).unwrap().clone();
        self.push(op::SET_VAR);
        self.push(var_pos);
    }

    pub fn push_constant(&mut self, constant: Constant) {
        let constant_pos = self.globals.get_constant(constant);
        self.push(constant_pos);
    }

    pub fn push(&mut self, opcode: u8) {
        self.bytecode.bytecode.push(opcode);
    }

    pub fn push_function_placeholder(&mut self, func: &SymbolFunc) {
        self.bytecode
            .call_placeholders
            .push((self.bytecode.bytecode.len(), func.clone()));
        self.push(0);
        self.push(0);
    }

    pub fn get_bytecode(&mut self) -> FunctionBytecode {
        let mut temp = FunctionBytecode { bytecode: vec![], call_placeholders: vec![] };
        std::mem::swap(&mut self.bytecode, &mut temp);
        temp
    }
}
