use std::collections::HashMap;

use crate::semantics::aggregate::ProgramAggregate;
use crate::semantics::symbols::SymbolFunc;

use self::generator::FunctionBytecode;

mod constants;
mod assemble;
pub mod disassemble;
mod expressions;
mod functions;
mod generator;
mod globals;

pub fn generate_program(prog: &ProgramAggregate) -> Vec<u8> {
    let mut globals = globals::Globals::new();

    let mut functions_bytecode: HashMap<SymbolFunc, FunctionBytecode> = HashMap::new();
    for (name, raw_func) in prog.functions.iter() {
        let bytecode = functions::generate_function_bytecode(raw_func, &prog, &mut globals).unwrap();
        functions_bytecode.insert(name.clone(), bytecode);
    }

    let const_bytecode = constants::constants_to_bytecode(&globals.constants);

    assemble::assemble_chunks(const_bytecode, functions_bytecode)
}
