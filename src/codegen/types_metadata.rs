use std::collections::HashMap;

use crate::semantics::annotations::CustomType;
use crate::semantics::symbols::SymbolType;

use super::utils::get_type_size;

pub struct TypeMetadata {
    pub size: u8,
    pub field_offsets: HashMap<String, u8>,
    pub field_sizes: HashMap<String, u8>,
}

pub struct TypeMetadataTable {
    indexes: HashMap<SymbolType, usize>,
    metadata: Vec<TypeMetadata>,
}

fn metadata_for_type(definition: &CustomType) -> TypeMetadata {
    let type_size: u8 = definition.fields.types.iter().map(|t| get_type_size(t)).sum();
    let field_sizes: Vec<u8> =
    definition.fields.types.iter().map(|t| get_type_size(t)).collect();
    let field_offsets = vec![0; field_sizes.len()];
    for (i, field_size) in field_sizes.iter().enumerate().skip(1) {
        field_offsets[i] = field_offsets[i - 1] + field_sizes[i - 1];
    }
    let generate_field_names = || definition.fields.iter().map(|(n, t)| n.clone());

    TypeMetadata {
        size: type_size,
        field_offsets: generate_field_names().zip(field_offsets).collect(),
        field_sizes: generate_field_names().zip(field_sizes).collect(),
    }
}

impl TypeMetadataTable {
    pub fn new(types: &HashMap<SymbolType, CustomType>) -> Self {
        let mut indexes = HashMap::new();
        let mut metadata = vec![];

        for (typename, definition) in types.iter() {
            let index = indexes.len();
            indexes.insert(typename.clone(), index);
            metadata.push(metadata_for_type(definition));
        }
        todo!("Test this");

        TypeMetadataTable { indexes, metadata }
    }

    pub fn get(&self, typename: &SymbolType) -> &TypeMetadata {
        &self.metadata[self.indexes[typename]]
    }
}


