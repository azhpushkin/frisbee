use std::collections::HashMap;

use super::heap::Heap;
use super::metadata::Metadata;

fn serialize_function_args(function_pos: usize, stack: &[u64], memory: &Heap, metadata: &Metadata) {
    // TODO: active objects should not be copied when serializing
    let func_index = metadata.function_positions[&function_pos];
    let locals_size = metadata.function_locals_sizes[func_index];

    let mut chunk: Vec<u64> = vec![0; locals_size];
    let mut pointers_to_pack: HashMap<u64, usize> = HashMap::new();
    let mut pointers_processed = 0;
    
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
                pos
            }
        };
        chunk[*pointer_index] = pos as u64;
    }
}