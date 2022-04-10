use crate::ast::Type;
use std::collections::HashMap;

pub struct CustomType<'a> {
    pub name: String,
    pub fields: TypedFields<'a>,
}

pub enum TrueType<'a> {
    Int,
    Float,
    String,
    Bool,
    Nil,

    List(Box<Self>),
    Tuple(Vec<Self>),
    Custom(&'a CustomType<'a>),
}

pub struct TypedFields<'a> {
    pub field_names: HashMap<String, usize>,
    pub field_types: Vec<TrueType<'a>>,
}

pub fn type_to_true<'a, 'b>(
    source_type: &'b Type,
    custom_resolver: &dyn Fn(&'b String) -> &'a CustomType,
) -> TrueType<'a> {
    match source_type {
        Type::Int => TrueType::Int,
        Type::Float => TrueType::Float,
        Type::Bool => TrueType::Bool,
        Type::Nil => TrueType::Nil,
        Type::String => TrueType::String,

        Type::List(inner) => {
            TrueType::List(Box::new(type_to_true(inner.as_ref(), custom_resolver)))
        }
        Type::Tuple(items) => {
            let true_items = items.iter().map(|t|type_to_true(t, custom_resolver));
            TrueType::Tuple(true_items.collect())
        }
        Type::Maybe(inner) => {
            let true_inner = type_to_true(inner.as_ref(), custom_resolver);
            TrueType::Tuple(vec![TrueType::Bool, true_inner])
        },
        Type::Ident(ident) => {
            TrueType::Custom(custom_resolver(ident))
        }
    }
}
