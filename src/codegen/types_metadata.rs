use std::collections::HashMap;

use crate::semantics::annotations::CustomType;
use crate::semantics::symbols::SymbolType;

use super::utils::get_type_size;

pub struct TypeMetadata {
    pub size: u8,
}

pub struct TypeMetadataTable {
    indexes: HashMap<SymbolType, usize>,
    metadata: Vec<TypeMetadata>,
}

impl TypeMetadataTable {
    pub fn new(types: &HashMap<SymbolType, CustomType>) -> Self {
        let mut indexes = HashMap::new();
        let mut metadata = vec![];

        for (typename, definiton) in types.iter() {
            let index = indexes.len();
            indexes.insert(typename.clone(), index);

            let type_size: u8 = definiton.fields.types.iter().map(|t| get_type_size(t)).sum();
            metadata.push(TypeMetadata { size: type_size });
        }

        TypeMetadataTable { indexes, metadata }
    }

    pub fn get_size(&self, typename: &SymbolType) -> u8 {
        self.metadata[self.indexes[typename]].size
    }
}
