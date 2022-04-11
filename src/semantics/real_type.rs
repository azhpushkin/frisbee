use crate::ast::{Type, TypedNamedObject};
use std::collections::HashMap;

use super::resolvers::SymbolResolver;

#[derive(Debug)]
pub struct CustomType {
    pub name: String,
    pub is_active: bool,
    pub fields: TypedFields,
}

#[derive(Debug, Clone)]
pub enum RType {
    Int,
    Float,
    String,
    Bool,

    List(Box<Self>),
    Tuple(Vec<Self>),
    Custom(String),
}

/// Simple ordered HashMap for typed and ordered fields
/// (used by function arguments and class types)
#[derive(Debug)]
pub struct TypedFields {
    pub names: HashMap<String, usize>,
    pub types: Vec<RType>,
}


pub fn type_to_real(source_type: &Type, custom_resolver: &SymbolResolver) -> RType {
    match source_type {
        Type::Int => RType::Int,
        Type::Float => RType::Float,
        Type::Bool => RType::Bool,
        Type::String => RType::String,

        Type::List(inner) => {
            RType::List(Box::new(type_to_real(inner.as_ref(), custom_resolver)))
        }
        Type::Tuple(items) => {
            let real_items = items.iter().map(|t| type_to_real(t, custom_resolver));
            RType::Tuple(real_items.collect())
        }
        Type::Maybe(inner) => {
            let real_inner = type_to_real(inner.as_ref(), custom_resolver);
            RType::Tuple(vec![RType::Bool, real_inner])
        }
        Type::Ident(ident) => RType::Custom(custom_resolver(ident).clone()),
    }
}

pub fn type_vec_to_typed_fields(
    v: &Vec<TypedNamedObject>,
    resolver: &SymbolResolver,
) -> TypedFields {
    let mut typed_fields = TypedFields { names: HashMap::new(), types: vec![] };

    for (i, old_type) in v.iter().enumerate() {
        let real_type = type_to_real(&old_type.typename, resolver);

        typed_fields.names.insert(old_type.name.clone(), i);
        typed_fields.types.push(real_type);
    }
    typed_fields
}
