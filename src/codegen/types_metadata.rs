use std::collections::HashMap;

use crate::semantics::annotations::CustomType;
use crate::semantics::symbols::SymbolType;

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
    let type_size: u8 = definition.fields.types.iter().map(|t| t.get_size()).sum();
    let field_sizes: Vec<u8> = definition.fields.types.iter().map(|t| t.get_size()).collect();

    let mut field_offsets = vec![0; field_sizes.len()];
    for (i, _) in field_sizes.iter().enumerate().skip(1) {
        field_offsets[i] = field_offsets[i - 1] + field_sizes[i - 1];
    }
    let generate_field_names = || definition.fields.iter().map(|(n, _)| n.clone());

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

        TypeMetadataTable { indexes, metadata }
    }

    pub fn get(&self, typename: &SymbolType) -> &TypeMetadata {
        &self.metadata[self.indexes[typename]]
    }
}

#[cfg(test)]
mod test {
    use crate::semantics::annotations::TypedFields;
    use crate::types::Type;

    use super::*;

    #[test]
    fn check_offsets_and_sizes() {
        let field_types = vec![
            Type::Int,
            Type::Tuple(vec![Type::Ident("Some".into()), Type::String]),
            Type::Bool,
        ];
        let field_names: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let fields = TypedFields {
            types: field_types,
            names: field_names.into_iter().enumerate().collect(),
        };
        let custom_type = CustomType {
            name: Type::Ident("module::MyType".into()).into(),
            is_active: false,
            fields,
        };

        let metadata = metadata_for_type(&custom_type);

        assert_eq!(metadata.size, 4);
        assert_eq!(metadata.field_offsets.len(), 3);
        assert_eq!(metadata.field_offsets["a".into()], 0);
        assert_eq!(metadata.field_offsets["b".into()], 1);
        assert_eq!(metadata.field_offsets["c".into()], 3);

        assert_eq!(metadata.field_sizes.len(), 3);
        assert_eq!(metadata.field_sizes["a".into()], 1);
        assert_eq!(metadata.field_sizes["b".into()], 2);
        assert_eq!(metadata.field_sizes["c".into()], 1);
    }
}
