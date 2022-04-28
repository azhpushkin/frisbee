use crate::ast::TypedNamedObject;
use crate::types::{verify_parsed_type, VerifiedType};
use std::collections::HashMap;

use super::resolvers::SymbolResolver;
use crate::symbols::SymbolType;

#[derive(Debug)]
pub struct CustomType {
    pub name: SymbolType,
    pub is_active: bool,
    pub fields: TypedFields,
}

/// Simple ordered HashMap for typed and ordered fields
/// (used by function arguments and class types)
#[derive(Debug)]
pub struct TypedFields {
    // TODO: remove pub, add methods for iter(), len() and add_this
    pub types: Vec<VerifiedType>,
    pub names: HashMap<usize, String>,
}

impl TypedFields {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &VerifiedType)> {
        self.types.iter().enumerate().map(move |(i, t)| (&self.names[&i], t))
    }
}

pub fn annotate_typednamed_vec(
    v: &[TypedNamedObject],
    resolver: &SymbolResolver<SymbolType>,
) -> Result<TypedFields, String> {
    let mut typed_fields = TypedFields { names: HashMap::new(), types: vec![] };

    for (i, old_type) in v.iter().enumerate() {
        let real_type = verify_parsed_type(&old_type.typename, resolver)?;

        typed_fields.names.insert(i, old_type.name.clone());
        typed_fields.types.push(real_type);
    }
    Ok(typed_fields)
}
