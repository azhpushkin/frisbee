use std::collections::HashMap;
use std::slice::Iter;

use crate::semantics::symbols::SymbolFunc;

use super::generator::FunctionBytecode;

pub fn assemble_chunks(
    mut constants: Vec<u8>,
    mut functions: HashMap<SymbolFunc, FunctionBytecode>,
) -> Vec<u8> {
    let mut functions_vec: Vec<SymbolFunc> = functions.keys().map(|x| x.clone()).collect();
    functions_vec.sort();

    let mut functions_start: HashMap<&SymbolFunc, usize> = HashMap::new();

    let mut current_shift = constants.len();
    for name in functions_vec.iter() {
        let bytecode = &functions[name].bytecode;
        functions_start.insert(name, current_shift);
        current_shift += bytecode.len();
    }
    for function in functions.values_mut() {
        let call_placeholders = function.call_placeholders.clone();

        for (pos, called_func) in call_placeholders {
            function.bytecode[pos] = functions_start[&called_func] as u8;
        }
    }

    for funcname in functions_vec.iter() {
        constants.extend(functions[funcname].bytecode.clone());
    }

    return constants;
}
