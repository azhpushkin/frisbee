use std::collections::HashMap;

use crate::ast::verified::{CustomType, RawFunction};
use crate::symbols::{SymbolFunc, SymbolType};
use crate::types::VerifiedType;

use self::generator::FunctionBytecode;

mod assemble;
mod constants;
mod disassemble;
mod expressions;
mod generator;
mod statements;
mod metadata;
mod utils;

pub fn generate(types: &[CustomType], functions: &[RawFunction], entry: &SymbolFunc) -> Vec<u8> {
    let mut constants = constants::ConstantsTable::new();
    let types_meta = metadata::TypesMetadataTable::from_types(types);
    let mut list_types_meta = metadata::ListMetadataTable::default();

    let mut functions_bytecode: HashMap<SymbolFunc, FunctionBytecode> = HashMap::new();
    for raw_function in functions.iter() {
        let mut bytecode = statements::generate_function_bytecode(
            raw_function,
            &types_meta,
            &mut list_types_meta,
            &mut constants,
        )
        .unwrap();
        bytecode.stack_pointer_mapping = utils::get_pointers_map_for_sequence(&raw_function.args.types);

        functions_bytecode.insert(raw_function.name.clone(), bytecode);
    }

    let constants_bytecode = constants.generate_bytecode();
    
    assemble::assemble_chunks(constants_bytecode, types_meta, list_types_meta, functions_bytecode, entry)
}


pub fn disassemble(program: &[u8]) -> String {
    let mut d = disassemble::Disassembler::new(program);
    d.disassemble()
}
