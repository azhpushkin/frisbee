use std::collections::HashMap;

use crate::ast::verified::{CustomType, RawFunction};
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

fn generate_chunks(
    types: &[&CustomType],
    functions: &[&RawFunction],
) -> (
    Vec<u8>,
    types_metadata::TypeMetadataTable,
    HashMap<SymbolFunc, FunctionBytecode>,
) {
    let mut constants = constants::ConstantsTable::new();
    let types_meta = types_metadata::TypeMetadataTable::new(types);
    let mut list_types_meta = types_metadata::TypeMetadataTable::new(&[]);

    let mut functions_bytecode: HashMap<SymbolFunc, FunctionBytecode> = HashMap::new();
    for raw_function in functions.iter() {
        let mut bytecode = statements::generate_function_bytecode(
            raw_function,
            &types_meta,
            &mut list_types_meta,
            &mut constants,
        )
        .unwrap();
        bytecode.stack_pointer_mapping = utils::generate_pointers_map(&raw_function.args.types);

        functions_bytecode.insert(raw_function.name.clone(), bytecode);
    }

    let constants_bytecode = constants.generate_bytecode();
    (constants_bytecode, types_meta, functions_bytecode)
}

pub fn generate(types: &[&CustomType], functions: &[&RawFunction], entry: &SymbolFunc) -> Vec<u8> {
    let (c, types_meta, f) = generate_chunks(types, functions);
    assemble::assemble_chunks(c, types_meta, f, entry)
}

pub fn disassemble(program: &[u8]) -> String {
    let mut d = disassemble::Disassembler::new(program);
    d.disassemble()
}
