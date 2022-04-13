use crate::semantics::aggregate::ProgramAggregate;
use crate::semantics::symbols::SymbolFunc;

use self::generator::FunctionBytecode;

mod constants;
pub mod disassemble;
mod expressions;
mod functions;
mod generator;
mod globals;

pub fn generate_program(prog: &ProgramAggregate) -> Vec<u8> {
    let mut globals = globals::Globals::new();

    let mut functions_bytecode: Vec<(&SymbolFunc, FunctionBytecode)> = vec![];
    for (name, raw_func) in prog.functions.iter() {
        let bytecode = functions::generate_function_bytecode(raw_func, &prog, &mut globals).unwrap();
        functions_bytecode.push((name, bytecode))
    }

    let const_bytecode = constants::constants_to_bytecode(&globals.constants);

    const_bytecode
}
