use std::collections::HashMap;

use super::heap::{Heap, HeapObject};
use super::metadata::Metadata;

pub const STRING_FLAG: u64 = 1 << 48;
pub const LIST_FLAG: u64 = 2 << 48;
pub const CUSTOM_OBJECT_FLAG: u64 = 4 << 48;

fn serialize_function_args(
    function_pos: usize,
    stack: &[u64],
    memory: &Heap,
    metadata: &Metadata,
) -> Vec<u64> {
    // TODO: active objects should not be copied when serializing
    let func_index = metadata.function_positions[&function_pos];
    let locals_size = metadata.function_locals_sizes[func_index];

    let mut chunk: Vec<u64> = vec![0; locals_size];
    let mut pointers_to_pack: HashMap<u64, usize> = HashMap::new();
    let mut pointers_order: Vec<u64> = vec![];
    let mut processed_amount = 0;

    // Prepare initial pointers for packing
    // Stack will not be used anymore as all the processing after this cycle is just
    // heap-data packing
    for pointer_index in metadata.functions_pointer_mapping[func_index].iter() {
        let heap_pointer = stack[*pointer_index];
        if heap_pointer == 0 {
            continue;
        }

        let pos = match pointers_to_pack.get(&heap_pointer) {
            Some(pos) => *pos,
            None => {
                let pos = pointers_to_pack.len() + 1;
                pointers_to_pack.insert(heap_pointer, pos);
                pointers_order.push(heap_pointer);
                pos
            }
        };
        chunk[*pointer_index] = pos as u64;
    }

    while processed_amount < pointers_order.len() {
        let pointer = pointers_order[processed_amount];
        processed_amount += 1;

        let heap_object = memory.get(pointer);

        chunk.push(get_heap_object_header(heap_object));
        let object_start = chunk.len();

        let pointer_map: &[usize] = match heap_object {
            HeapObject::String(s) => {
                // TODO: this needs to be refactored to make chars "tighter"
                chunk.extend(s.chars().map(|c| c as u64));
                &[]
            }
            HeapObject::List(l) => {
                chunk.extend(l.data.iter());
                &metadata.lists_pointer_mapping[l.list_item_type]
            }
            HeapObject::CustomObject(obj) => {
                chunk.extend(obj.data.iter());
                &metadata.types_pointer_mapping[obj.type_index as usize]
            }
        };

        for offset in pointer_map.iter().map(|i| *i + object_start) {
            if chunk[offset] == 0 {
                continue;
            }
            let heap_pointer = chunk[offset];

            let pos = match pointers_to_pack.get(&heap_pointer) {
                Some(pos) => *pos,
                None => {
                    let pos = pointers_to_pack.len() + 1;
                    pointers_to_pack.insert(heap_pointer, pos);
                    pointers_order.push(heap_pointer);
                    pos
                }
            };
            chunk[offset] = pos as u64;
        }
    }
    chunk
}

fn get_heap_object_header(obj: &HeapObject) -> u64 {
    let obj_header: u64;

    match obj {
        HeapObject::String(s) => {
            obj_header = s.len() as u64 | STRING_FLAG;
        }
        HeapObject::List(l) => {
            obj_header = l.items_amount as u64 | LIST_FLAG;
        }
        HeapObject::CustomObject(obj) => {
            obj_header = 0 | CUSTOM_OBJECT_FLAG;
        }
    }
    obj_header
}
