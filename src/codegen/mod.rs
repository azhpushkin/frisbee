use crate::ast::verified::{CustomType, RawFunction};
use crate::symbols::SymbolFunc;

use self::generator::FunctionBytecode;

mod assemble;
mod constants;
mod disassemble;
mod expressions;
mod generator;
mod metadata;
mod statements;
mod utils;

pub fn generate(types: &[CustomType], functions: &[RawFunction], entry: &SymbolFunc) -> Vec<u8> {
    let mut constants = constants::ConstantsTable::new();
    let custom_types_meta = metadata::CustomTypesMetadataTable::from_types(types);
    let mut list_kinds_meta = metadata::ListKindsMetadataTable::new_empty();

    let mut functions_bytecode: Vec<FunctionBytecode> = vec![];
    for raw_function in functions.iter() {
        let mut bytecode = statements::generate_function_bytecode(
            raw_function,
            &custom_types_meta,
            &mut list_kinds_meta,
            &mut constants,
        )
        .unwrap();
        bytecode.args_pointer_mapping =
            utils::get_pointers_map_for_sequence(&raw_function.args.types);

        functions_bytecode.push(bytecode);
    }

    let constants_bytecode = constants.generate_bytecode();

    assemble::assemble_chunks(
        constants_bytecode,
        custom_types_meta,
        list_kinds_meta,
        functions_bytecode,
        entry,
    )
}

pub fn disassemble(program: &[u8]) -> String {
    let mut d = disassemble::Disassembler::new(program);
    d.disassemble()
}
