use std::collections::HashMap;

use crate::ast::verified::CustomType;
use crate::symbols::SymbolType;
use crate::types::VerifiedType;

use super::utils;

#[derive(Default)]
pub struct TypeMetadata {
    pub size: u8,
    pub field_offsets: HashMap<String, u8>,
    pub field_sizes: HashMap<String, u8>,
    pub pointer_mapping: Vec<usize>,
}

#[derive(Default)]
pub struct ListItemMetadata {
    pub size: u8,
    pub pointer_mapping: Vec<usize>,
}

#[derive(Default)]
pub struct TypesMetadataTable {
    pub indexes: HashMap<SymbolType, usize>,
    pub metadata: Vec<TypeMetadata>,
}

#[derive(Default)]
pub struct ListMetadataTable {
    pub indexes: HashMap<VerifiedType, usize>,
    pub metadata: Vec<ListItemMetadata>,
}

impl TypeMetadata {
    pub fn from_custom(definition: &CustomType) -> Self {
        let fields_sizes = definition.fields.types.iter().map(utils::get_type_size);
        let type_size: u8 = fields_sizes.clone().sum();
        let field_sizes: Vec<u8> = fields_sizes.collect();

        let mut field_offsets = vec![0; field_sizes.len()];
        for (i, _) in field_sizes.iter().enumerate().skip(1) {
            field_offsets[i] = field_offsets[i - 1] + field_sizes[i - 1];
        }
        let generate_field_names = || definition.fields.iter().map(|(n, _)| n.clone());

        Self {
            size: type_size,
            field_offsets: generate_field_names().zip(field_offsets).collect(),
            field_sizes: generate_field_names().zip(field_sizes).collect(),
            pointer_mapping: utils::get_pointers_map_for_sequence(&definition.fields.types),
        }
    }
}

impl TypesMetadataTable {
    pub fn from_types(types: &[CustomType]) -> Self {
        let mut table = Self::default();

        for custom_type in types.iter() {
            let index = table.indexes.len();
            table.indexes.insert(custom_type.name.clone(), index);
            table.metadata.push(TypeMetadata::from_custom(custom_type));
        }

        table
    }

    pub fn get_meta(&self, flag: &SymbolType) -> &TypeMetadata {
        &self.metadata[self.indexes[flag]]
    }

    pub fn get_index(&self, flag: &SymbolType) -> usize {
        self.indexes[flag]
    }
}

impl ListItemMetadata {
    pub fn from_type(t: &VerifiedType) -> Self {
        Self {
            size: utils::get_type_size(t),
            pointer_mapping: utils::get_pointers_map_for_type(t),
        }
    }
}

impl ListMetadataTable {
    pub fn get_or_insert(&mut self, t: &VerifiedType) -> usize {
        if let Some(index) = self.indexes.get(t) {
            return *index;
        } else {
            let index = self.indexes.len();
            self.metadata.push(ListItemMetadata::from_type(t));
            self.indexes.insert(t.clone(), index);
            index
        }
    }
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

        let metadata = TypeMetadata::from_custom(&custom_type);

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
