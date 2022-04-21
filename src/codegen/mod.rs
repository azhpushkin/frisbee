use std::collections::HashMap;

use crate::semantics::aggregate::ProgramAggregate;
use crate::semantics::symbols::SymbolFunc;

use self::generator::FunctionBytecode;

mod assemble;
mod constants;
mod disassemble;
mod expressions;
mod statements;
mod generator;
mod globals;

fn generate_chunks(prog: &ProgramAggregate) -> (Vec<u8>, HashMap<SymbolFunc, FunctionBytecode>) {
    let mut globals = globals::Globals::new();

    let mut functions_bytecode: HashMap<SymbolFunc, FunctionBytecode> = HashMap::new();
    for (name, raw_func) in prog.functions.iter() {
        let bytecode =
            statements::generate_function_bytecode(raw_func, &prog, &mut globals).unwrap();
        functions_bytecode.insert(name.clone(), bytecode);
    }

    let const_bytecode = constants::constants_to_bytecode(&globals.constants);
    (const_bytecode, functions_bytecode)
}

pub fn generate(prog: &ProgramAggregate) -> Vec<u8> {
    let (c, f) = generate_chunks(prog);
    assemble::assemble_chunks(c, f, &prog.entry)
}

pub fn disassemble(program: &Vec<u8>) -> String {
    let mut d = disassemble::Disassembler::new(&program);
    return d.disassemble();
}
