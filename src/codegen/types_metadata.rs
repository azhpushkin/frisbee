use std::collections::HashMap;

use crate::ast::verified::CustomType;
use crate::symbols::SymbolType;

use super::utils::{get_type_size, generate_pointers_map};

pub struct TypeMetadata {
    pub size: u8,
    pub field_offsets: HashMap<String, u8>,
    pub field_sizes: HashMap<String, u8>,
    pub pointer_mapping: Vec<usize>
}

#[derive(Default)]
pub struct TypeMetadataTable<T> where T: std::hash::Hash + Eq {
    pub indexes: HashMap<T, usize>,
    pub metadata: Vec<TypeMetadata>,
}

fn metadata_for_type(definition: &CustomType) -> TypeMetadata {
    let type_size: u8 = definition.fields.types.iter().map(get_type_size).sum();
    let field_sizes: Vec<u8> = definition.fields.types.iter().map(get_type_size).collect();

    let mut field_offsets = vec![0; field_sizes.len()];
    for (i, _) in field_sizes.iter().enumerate().skip(1) {
        field_offsets[i] = field_offsets[i - 1] + field_sizes[i - 1];
    }
    let generate_field_names = || definition.fields.iter().map(|(n, _)| n.clone());

    TypeMetadata {
        size: type_size,
        field_offsets: generate_field_names().zip(field_offsets).collect(),
        field_sizes: generate_field_names().zip(field_sizes).collect(),
        pointer_mapping: generate_pointers_map(&definition.fields.types),
    }
}

impl<T> TypeMetadataTable<T> where T: std::hash::Hash + Eq {
    pub fn get(&self, flag: &T) -> &TypeMetadata {
        &self.metadata[self.indexes[flag]]
    }

    pub fn new() -> Self {
        TypeMetadataTable{
            indexes: HashMap::new(),
            metadata: vec![],
        }
    }
}

pub fn meta_table_for_types(types: &[&CustomType]) -> TypeMetadataTable<SymbolType> {
    let mut table = TypeMetadataTable::new();

    for custom_type in types.iter() {
        let index = table.indexes.len();
        table.indexes.insert(custom_type.name.clone(), index);
        table.metadata.push(metadata_for_type(custom_type));
    }

    table
}

#[cfg(test)]
mod test {
    use crate::alias::ModuleAlias;
    use crate::ast::verified::TypedFields;
    use crate::types::Type;

    use super::*;

    #[test]
    fn check_offsets_and_sizes() {
        let module_alias = ModuleAlias::new(&["somemodule".into()]);
        let gen_symbol_type = || SymbolType::new(&module_alias, "Type");
        let field_types = vec![
            Type::Int,
            Type::Tuple(vec![Type::Custom(gen_symbol_type()), Type::String]),
            Type::Bool,
        ];
        let field_names: Vec<String> = vec!["a".into(), "b".into(), "c".into()];
        let fields = TypedFields {
            types: field_types,
            names: field_names.into_iter().enumerate().collect(),
        };
        let custom_type = CustomType { name: gen_symbol_type(), is_active: false, fields };

        let metadata = metadata_for_type(&custom_type);

        assert_eq!(metadata.size, 4);
        assert_eq!(metadata.field_offsets.len(), 3);
        assert_eq!(metadata.field_offsets["a"], 0);
        assert_eq!(metadata.field_offsets["b"], 1);
        assert_eq!(metadata.field_offsets["c"], 3);

        assert_eq!(metadata.field_sizes.len(), 3);
        assert_eq!(metadata.field_sizes["a"], 1);
        assert_eq!(metadata.field_sizes["b"], 2);
        assert_eq!(metadata.field_sizes["c"], 1);
    }
}
