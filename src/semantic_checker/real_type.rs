use crate::ast::{Type, TypedNamedObject};
use std::collections::HashMap;

#[derive(Debug)]
pub struct CustomType {
    pub name: String,
    pub fields: TypedFields,
}

#[derive(Debug, Clone)]
pub enum RType {
    Int,
    Float,
    String,
    Bool,
    Nil,

    List(Box<Self>),
    Tuple(Vec<Self>),
    Custom(String),
}

#[derive(Debug)]
pub struct TypedFields {
    pub names: HashMap<String, usize>,
    pub types: Vec<RType>,
}

pub type CustomResolver = dyn Fn(&String) -> &String;

pub fn type_to_real(source_type: &Type, custom_resolver: &CustomResolver) -> RType {
    match source_type {
        Type::Int => RType::Int,
        Type::Float => RType::Float,
        Type::Bool => RType::Bool,
        Type::Nil => RType::Nil,
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
    resolver: &CustomResolver,
) -> TypedFields {
    let mut typed_fields = TypedFields { names: HashMap::new(), types: vec![] };

    for (i, old_type) in v.iter().enumerate() {
        let real_type = type_to_real(&old_type.typename, resolver);

        typed_fields.names.insert(old_type.name.clone(), i);
        typed_fields.types.push(real_type);
    }
    typed_fields
}
