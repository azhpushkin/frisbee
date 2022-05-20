use std::collections::HashMap;

use crate::vm::heap::{CustomObject, List};

use super::heap::{Heap, HeapObject};
use super::metadata::Metadata;

pub const STRING_FLAG: u64 = 1 << 56;
pub const LIST_FLAG: u64 = 2 << 56;
pub const CUSTOM_OBJECT_FLAG: u64 = 4 << 56;

pub fn serialize_function_args(
    function_pos: usize,
    stack: &[u64],
    stack_pointer: &mut usize,
    heap: &Heap,
    metadata: &Metadata,
) -> Vec<u64> {
    // TODO: active objects should not be copied when serializing
    let func_index = metadata.function_positions[&function_pos];
    let locals_size = metadata.function_locals_sizes[func_index];
    *stack_pointer -= locals_size;
    // println!("Serializing {:?} for locals size {}", &stack[..10], locals_size);

    let mut chunk: Vec<u64> = vec![function_pos as u64];
    chunk.extend(stack.iter().skip(*stack_pointer).take(locals_size));
    let mut pointers_to_pack: HashMap<u64, usize> = HashMap::new();
    let mut pointers_order: Vec<u64> = vec![];
    let mut processed_amount = 0;

    // Prepare initial pointers for packing
    // Stack will not be used anymore as all the processing after this cycle is just
    // heap-data packing
    for pointer_index in metadata.functions_pointer_mapping[func_index].iter() {
        let heap_pointer = chunk[*pointer_index + 1];
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
        chunk[*pointer_index + 1] = pos as u64;
    }

    while processed_amount < pointers_order.len() {
        let pointer = pointers_order[processed_amount];
        processed_amount += 1;

        let heap_object = heap.get(pointer);

        chunk.push(serialize_heap_object_header(heap_object));
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
    // println!("  chunk is {:?}", chunk);
    chunk
}

fn serialize_heap_object_header(obj: &HeapObject) -> u64 {
    let obj_header: u64;

    match obj {
        HeapObject::String(s) => {
            obj_header = s.len() as u64 | STRING_FLAG;
        }
        HeapObject::List(l) => {
            obj_header = (l.list_item_type as u64) << 32 | (l.items_amount as u64) | LIST_FLAG;
        }
        HeapObject::CustomObject(obj) => {
            obj_header = obj.type_index | CUSTOM_OBJECT_FLAG;
        }
    }
    obj_header
}

pub fn deserialize_function_args(
    function_pos: usize,
    stack: &mut [u64],
    stack_pointer: &mut usize,
    heap: &mut Heap,
    metadata: &Metadata,
    chunk: &Vec<u64>,
) {
    assert_eq!(chunk[0], function_pos as u64, "wrong function extracted");
    // println!("deserializing {:?}", chunk);

    let func_index = metadata.function_positions[&function_pos];
    let locals_size = metadata.function_locals_sizes[func_index];

    for i in 0..locals_size {
        stack[*stack_pointer + i] = chunk[i + 1];
    }
    *stack_pointer += locals_size;

    let mut heap_pointers_to_fill: Vec<(u64, &Vec<usize>)> = vec![];

    let mut heap_objects_mapping: HashMap<usize, u64> = HashMap::new();

    let mut current_start = locals_size + 1;
    while current_start < chunk.len() {
        let obj_header = chunk[current_start];

        if (obj_header & STRING_FLAG) != 0 {
            let string_length = (obj_header & !STRING_FLAG) as usize;
            // TODO: check utf-8 probably
            let parsed_string = chunk[current_start + 1..current_start + 1 + string_length]
                .iter()
                .map(|c| char::from_u32(*c as u32).unwrap())
                .collect::<String>();
            let (pos, _) = heap.move_string(parsed_string);
            heap_objects_mapping.insert(heap_objects_mapping.len() + 1, pos);
            current_start += 1 + string_length;
        } else if (obj_header & LIST_FLAG) != 0 {
            let obj_header = obj_header & !LIST_FLAG;
            let list_type = obj_header >> 32;
            let list_items_amount = (obj_header & !list_type) as usize; // TODO: improve this, 0fff or smth like this

            let (pos, l) = heap.allocate_list(
                list_type as usize,
                list_items_amount,
                &chunk[current_start + 1..],
                metadata,
            );
            heap_objects_mapping.insert(heap_objects_mapping.len() + 1, pos);
            current_start += 1 + l.data.len();

            heap_pointers_to_fill.push((pos, &metadata.lists_pointer_mapping[list_type as usize]));
        } else if (obj_header & CUSTOM_OBJECT_FLAG) != 0 {
            let obj_type = obj_header & !CUSTOM_OBJECT_FLAG;
            let (pos, new_obj) = heap.allocate_custom(obj_type as usize, metadata);
            new_obj.data.clone_from_slice(&chunk[current_start + 1..]);
            heap_objects_mapping.insert(heap_objects_mapping.len() + 1, pos);
            current_start += 1 + new_obj.data.len();

            heap_pointers_to_fill.push((pos, &metadata.types_pointer_mapping[obj_type as usize]));
        } else {
            panic!("I have no idea what this {} flag is about..", obj_header);
        }
    }

    // Now fill all the pointers
    // Start with the stack
    for pointer in metadata.functions_pointer_mapping[func_index].iter() {
        let stack_value = stack[*pointer] as usize;
        if stack_value != 0 {
            stack[*pointer] = heap_objects_mapping[&stack_value];
        }
    }

    // And proceed with a heap
    for (heap_obj_pointer, pointers) in heap_pointers_to_fill {
        let heap_obj = heap.get_mut(heap_obj_pointer);
        let data = match heap_obj {
            HeapObject::String(_) => unreachable!(),
            HeapObject::List(List { data, .. }) => data,
            HeapObject::CustomObject(CustomObject { data, .. }) => data,
        };
        for pointer in pointers.iter() {
            let value = data[*pointer] as usize;
            if value != 0 {
                data[*pointer] = heap_objects_mapping[&value];
            }
        }
    }
}
