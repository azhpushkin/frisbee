use std::collections::HashMap;

use crate::ast::ModulePathAlias;
use crate::vm::op;

use super::constants::Constant;
use super::globals::Globals;

pub struct BytecodeGenerator<'a> {
    globals: &'a mut Globals,
    locals: HashMap<&'a String, u8>,
    bytecode: Vec<u8>,
}

impl<'a> BytecodeGenerator<'a> {
    pub fn new(globals: &'a mut Globals, locals: HashMap<&'a String, u8>) -> Self {
        BytecodeGenerator { globals, locals, bytecode: vec![] }
    }

    pub fn add_local(&mut self, varname: &'a String) {
        self.locals.insert(varname, self.locals.len() as u8);
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

    pub fn get_function(&mut self, module: &ModulePathAlias, function: &String) -> usize {
        self.globals.get_function_placeholder(module.clone(), function.clone())
    }

    pub fn push(&mut self, opcode: u8) {
        self.bytecode.push(opcode);
    }

    pub fn get_bytecode(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.bytecode)
    }
}
