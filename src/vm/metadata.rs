use std::collections::HashMap;

pub type MetadataBlock = Vec<(usize, Vec<u8>)>;

#[derive(Default)]
pub struct Metadata {
    pub types_sizes: Vec<u8>,
    pub list_types_sizes: Vec<u8>,

    pub types_pointer_mapping: Vec<Vec<u8>>,
    pub lists_pointer_mapping: Vec<Vec<u8>>,
    pub functions_pointer_mapping: HashMap<usize, Vec<u8>>,
}

impl Metadata {
    pub fn fill_types_metadata(&mut self, types_metadata: MetadataBlock) {
        for (size, mapping) in types_metadata {
            self.types_sizes.push(size as u8);
            self.types_pointer_mapping.push(mapping);
        }
    }

    pub fn fill_lists_metadata(&mut self, lists_metadata: MetadataBlock) {
        for (size, mapping) in lists_metadata {
            self.list_types_sizes.push(size as u8);
            self.lists_pointer_mapping.push(mapping);
        }
    }

    pub fn fill_function_metadata(&mut self, funcs_metadata: MetadataBlock) {
        for (pos, mapping) in funcs_metadata {
            self.functions_pointer_mapping.insert(pos, mapping);
        }
    }
}
