use std::collections::HashMap;
use std::slice::Iter;

use crate::semantics::symbols::SymbolFunc;

use super::generator::FunctionBytecode;

pub fn assemble_chunks(
    mut constants: Vec<u8>,
    mut functions: HashMap<SymbolFunc, FunctionBytecode>,
    entry: &SymbolFunc,
) -> Vec<u8> {
    let mut functions_vec: Vec<SymbolFunc> = functions.keys().map(|x| x.clone()).collect();
    functions_vec.sort();

    let mut functions_start: HashMap<&SymbolFunc, usize> = HashMap::new();

    constants.extend([0, 0]); // 2 bytes to encode entry function pos

    let mut current_shift = constants.len();

    for name in functions_vec.iter() {
        let bytecode = &functions[name].bytecode;
        functions_start.insert(name, current_shift);
        current_shift += bytecode.len();
    }
    let entry_pos = (functions_start[&entry] as u16).to_be_bytes();
    let x = constants.len();
    constants[x - 2] = entry_pos[0];
    constants[x - 1] = entry_pos[1];

    for function in functions.values_mut() {
        let call_placeholders = function.call_placeholders.clone();

        for (pos, called_func) in call_placeholders {
            let start = (functions_start[&called_func] as u16).to_be_bytes();
            function.bytecode[pos] = start[0];
            function.bytecode[pos + 1] = start[1];
        }
    }

    for funcname in functions_vec.iter() {
        constants.extend(functions[funcname].bytecode.clone());
    }

    return constants;
}
