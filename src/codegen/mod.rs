use std::collections::HashMap;

use crate::semantics::aggregate::ProgramAggregate;
use crate::symbols::SymbolFunc;

use self::generator::FunctionBytecode;

mod assemble;
mod constants;
mod disassemble;
mod expressions;
mod generator;
mod statements;
mod types_metadata;
mod utils;

fn generate_chunks(prog: &ProgramAggregate) -> (Vec<u8>, HashMap<SymbolFunc, FunctionBytecode>) {
    let mut constants = constants::ConstantsTable::new();
    let types_metadata = types_metadata::TypeMetadataTable::new(&prog.types);

    let mut functions_bytecode: HashMap<SymbolFunc, FunctionBytecode> = HashMap::new();
    for (name, raw_func) in prog.functions.iter() {
        let bytecode =
            statements::generate_function_bytecode(raw_func, &types_metadata, &mut constants)
                .unwrap();
        functions_bytecode.insert(name.clone(), bytecode);
    }

    let constants_bytecode = constants.generate_bytecode();
    (constants_bytecode, functions_bytecode)
}

pub fn generate(prog: &ProgramAggregate) -> Vec<u8> {
    let (c, f) = generate_chunks(prog);
    assemble::assemble_chunks(c, f, &prog.entry)
}

pub fn disassemble(program: &Vec<u8>) -> String {
    let mut d = disassemble::Disassembler::new(&program);
    return d.disassemble();
}
