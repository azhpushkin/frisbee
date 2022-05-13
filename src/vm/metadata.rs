use std::collections::HashMap;

pub type MetadataBlock = Vec<(usize, Vec<u8>)>;

#[derive(Default)]
pub struct Metadata {
    pub types_sizes: Vec<usize>,
    pub list_types_sizes: Vec<usize>,
    pub function_locals_sizes: Vec<usize>,

    pub types_pointer_mapping: Vec<Vec<usize>>,
    pub lists_pointer_mapping: Vec<Vec<usize>>,
    pub functions_pointer_mapping: Vec<Vec<usize>>,

    pub function_positions: HashMap<usize, usize>, // bytecode position -> function index
}

impl Metadata {
    pub fn fill_types_metadata(&mut self, types_metadata: MetadataBlock) {
        for (size, mapping) in types_metadata {
            self.types_sizes.push(size);
            self.types_pointer_mapping
                .push(mapping.into_iter().map(|x| x as usize).collect());
        }
    }

    pub fn fill_lists_metadata(&mut self, lists_metadata: MetadataBlock) {
        for (size, mapping) in lists_metadata {
            self.list_types_sizes.push(size);
            self.lists_pointer_mapping
                .push(mapping.into_iter().map(|x| x as usize).collect());
        }
    }

    pub fn fill_function_metadata(&mut self, funcs_metadata: MetadataBlock) {
        for (size, mapping) in funcs_metadata {
            self.function_locals_sizes.push(size);
            self.functions_pointer_mapping
                .push(mapping.into_iter().map(|x| x as usize).collect());
        }
    }
}
