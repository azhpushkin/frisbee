use std::collections::HashMap;

use crate::symbols::{SymbolFunc, SymbolType};
use crate::types::VerifiedType;

use super::generator::FunctionBytecode;
use super::metadata::{TypesMetadataTable, ListMetadataTable};

/*
Bytecode structure:
 - 0xff 0xff : two starting bytes
 - constants block (see constants.rs::constants_to_bytecode)
    - constants block ends with CONST_END_FLAG byte
 - symbols info block, that contains function names
    - each block starts with a string (2 bytes for length + string)
    - then, placeholder for the function start
 - functions bytecode

*/

static HEADER: [u8; 2] = [u8::MAX, u8::MAX];

pub fn assemble_chunks(
    constants: Vec<u8>,
    types_meta: TypesMetadataTable,
    list_types_meta: ListMetadataTable,
    functions: HashMap<SymbolFunc, FunctionBytecode>,
    entry: &SymbolFunc,
) -> Vec<u8> {
    // 1. Initial header
    let mut bytecode: Vec<u8> = HEADER.into();

    // 2. constants block along with trailing header
    bytecode.extend(constants);
    bytecode.extend_from_slice(&HEADER);

    // 3. Types info (size + pointer mapping)
    bytecode.push(types_meta.metadata.len() as u8);
    for type_meta in types_meta.metadata.iter() {
        bytecode.push(type_meta.size);
        bytecode.push(type_meta.pointer_mapping.len() as u8);
        bytecode.extend(type_meta.pointer_mapping.iter().map(|x| *x as u8));
    }
    bytecode.extend_from_slice(&HEADER);

    // 4. Symbol names block
    let mut encoded_symbols_info: HashMap<usize, &SymbolFunc> = HashMap::new();

    for (function_name, function_bytecode) in functions.iter() {
        let name_s: String = format!("{}", function_name);
        bytecode.extend((name_s.len() as u16).to_be_bytes());
        bytecode.extend(name_s.as_bytes());

        encoded_symbols_info.insert(bytecode.len(), function_name);
        bytecode.extend([0, 0]);
    }

    // end of symbols info marked with 0, 0, 255, 255
    bytecode.extend([0, 0]);
    bytecode.extend_from_slice(&HEADER);

    // 5. Entry function pointer + header
    encoded_symbols_info.insert(bytecode.len(), entry);
    bytecode.extend([0, 0]);
    bytecode.extend_from_slice(&HEADER);

    // 6. Functions bytecode, no headers anymore
    let mut functions_start: HashMap<&SymbolFunc, usize> = HashMap::new();

    for (name, function_bytecode) in functions.iter() {
        for (pos, called_func) in function_bytecode.call_placeholders.iter() {
            encoded_symbols_info.insert(*pos + bytecode.len(), called_func);
        }
        functions_start.insert(name, bytecode.len());
        bytecode.extend_from_slice(&function_bytecode.bytecode);
    }

    // BACKTRACKING: fill function pointers in CALL operations and symbol table
    for (pos, called_func) in encoded_symbols_info.iter() {
        let start = (functions_start[called_func] as u16).to_be_bytes();
        bytecode[*pos] = start[0];
        bytecode[*pos + 1] = start[1];
    }

    bytecode
}
