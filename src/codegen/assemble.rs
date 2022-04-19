use std::collections::HashMap;
use std::slice::Iter;

use crate::semantics::symbols::SymbolFunc;

use super::generator::FunctionBytecode;

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

pub fn assemble_chunks(
    constants: Vec<u8>,
    functions: HashMap<SymbolFunc, FunctionBytecode>,
    entry: &SymbolFunc,
) -> Vec<u8> {
    let mut bytecode: Vec<u8> = vec![u8::MAX, u8::MAX]; // initial flag
    bytecode.extend(constants);
    bytecode.extend(vec![u8::MAX, u8::MAX]);

    let mut encoded_symbols_info: HashMap<usize, &SymbolFunc> = HashMap::new();

    for function_name in functions.keys() {
        let name_s: String = function_name.into();
        bytecode.extend((name_s.len() as u16).to_be_bytes());
        bytecode.extend(name_s.as_bytes());

        encoded_symbols_info.insert(bytecode.len(), function_name);
        bytecode.extend([0, 0]);
    }

    bytecode.extend([0, 0, u8::MAX, u8::MAX]); // end of symbols info

    // 2 bytes to encode entry function pos
    encoded_symbols_info.insert(bytecode.len(), entry);
    bytecode.extend([0, 0]);

    let mut functions_start: HashMap<&SymbolFunc, usize> = HashMap::new();

    for (name, function_bytecode) in functions.iter() {
        for (pos, called_func) in function_bytecode.call_placeholders.iter() {
            encoded_symbols_info.insert(*pos + bytecode.len(), called_func);
        }
        functions_start.insert(name, bytecode.len());
        bytecode.extend_from_slice(&function_bytecode.bytecode);
    }

    for (pos, called_func) in encoded_symbols_info.iter() {
        let start = (functions_start[called_func] as u16).to_be_bytes();
        bytecode[*pos] = start[0];
        bytecode[*pos + 1] = start[1];
    }

    return bytecode;
}
