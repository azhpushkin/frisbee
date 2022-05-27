use std::collections::HashMap;

use crate::symbols::SymbolFunc;

use super::generator::FunctionBytecode;
use super::metadata::{CustomTypesMetadataTable, ListKindsMetadataTable};

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

fn get_by_value<'a, K>(m: &'a HashMap<K, usize>, value: usize) -> &'a K {
    m.iter().find(|(_, v)| **v == value).unwrap().0
}

fn push_str(bytecode: &mut Vec<u8>, s: &str) {
    bytecode.extend((s.len() as u16).to_be_bytes());
    bytecode.extend(s.as_bytes());
}

fn push_pointers_map(bytecode: &mut Vec<u8>, pointers_map: &[usize]) {
    bytecode.push(pointers_map.len() as u8);
    bytecode.extend(pointers_map.iter().map(|x| *x as u8));
}

fn push_usize_as_u16(bytecode: &mut Vec<u8>, value: usize) {
    bytecode.extend((value as u16).to_be_bytes());
}

pub fn assemble_chunks(
    constants: Vec<u8>,
    custom_types_meta: CustomTypesMetadataTable,
    list_kinds_meta: ListKindsMetadataTable,
    functions: Vec<FunctionBytecode>,
    entry: &SymbolFunc,
) -> Vec<u8> {
    // 1. Initial header
    let mut bytecode: Vec<u8> = HEADER.into();

    // 2. constants block along with trailing header
    bytecode.extend(constants);
    bytecode.extend_from_slice(&HEADER);

    // 3. Types info (size + pointer mapping)
    bytecode.push(custom_types_meta.metadata.len() as u8);
    for (i, type_meta) in custom_types_meta.metadata.iter().enumerate() {
        let type_name = get_by_value(&custom_types_meta.indexes, i);
        push_str(&mut bytecode, &format!("{}", type_name));
        push_usize_as_u16(&mut bytecode, type_meta.size as usize);
        push_pointers_map(&mut bytecode, &type_meta.pointer_mapping);
    }
    bytecode.extend_from_slice(&HEADER);

    // 4. List kinds info (item size + pointer mapping)
    bytecode.push(list_kinds_meta.metadata.len() as u8);
    for (i, list_kind_meta) in list_kinds_meta.metadata.iter().enumerate() {
        let item_type = get_by_value(&list_kinds_meta.indexes, i);
        push_str(&mut bytecode, &format!("{}", item_type));
        push_usize_as_u16(&mut bytecode, list_kind_meta.size as usize);
        push_pointers_map(&mut bytecode, &list_kind_meta.pointer_mapping);
    }
    bytecode.extend_from_slice(&HEADER);

    // 5. Functions info (names + locals sizes + pointer mapping)
    bytecode.push(functions.len() as u8);
    for function_info in functions.iter() {
        push_str(&mut bytecode, &format!("{}", function_info.name));
        push_usize_as_u16(&mut bytecode, function_info.args_size);
        push_pointers_map(&mut bytecode, &function_info.args_pointer_mapping);
    }
    bytecode.extend_from_slice(&HEADER);

    // 6. Function positions (debug info)
    let mut encoded_symbols_info: HashMap<usize, &SymbolFunc> = HashMap::new();
    for function_info in functions.iter() {
        encoded_symbols_info.insert(bytecode.len(), &function_info.name);
        bytecode.extend([0, 0]);
    }
    bytecode.extend_from_slice(&HEADER);

    // 7. Entry function pointer + header
    encoded_symbols_info.insert(bytecode.len(), entry);
    bytecode.extend([0, 0]); // placeholder, will be filled in later
    bytecode.extend_from_slice(&HEADER);

    // 8. Functions bytecode, no headers anymore
    let mut functions_start: HashMap<&SymbolFunc, usize> = HashMap::new();

    for function_bytecode in functions.iter() {
        for (pos, called_func) in function_bytecode.call_placeholders.iter() {
            encoded_symbols_info.insert(*pos + bytecode.len(), called_func);
        }
        functions_start.insert(&function_bytecode.name, bytecode.len());
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
