use crate::ast::{Type, TypedNamedObject};
use std::collections::HashMap;

use super::resolvers::{SymbolResolver};
use super::symbols::SymbolType;

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
    pub names: HashMap<String, usize>,
    pub types: Vec<Type>,
}


pub fn annotate_type(source_type: &Type, custom_resolver: &SymbolResolver<SymbolType>) -> Type {
    match source_type {
        Type::Int => Type::Int,
        Type::Float => Type::Float,
        Type::Bool => Type::Bool,
        Type::String => Type::String,

        Type::List(inner) => {
            Type::List(Box::new(annotate_type(inner.as_ref(), custom_resolver)))
        }
        Type::Tuple(items) => {
            let real_items = items.iter().map(|t| annotate_type(t, custom_resolver));
            Type::Tuple(real_items.collect())
        }
        Type::Maybe(inner) => {
            let real_inner = annotate_type(inner.as_ref(), custom_resolver);
            Type::Tuple(vec![Type::Bool, real_inner])
        }
        Type::Ident(ident) => custom_resolver(ident).into(),
    }
}

pub fn annotate_typednamed_vec(
    v: &Vec<TypedNamedObject>,
    resolver: &SymbolResolver<SymbolType>,
) -> TypedFields {
    let mut typed_fields = TypedFields { names: HashMap::new(), types: vec![] };

    for (i, old_type) in v.iter().enumerate() {
        let real_type = annotate_type(&old_type.typename, resolver);

        typed_fields.names.insert(old_type.name.clone(), i);
        typed_fields.types.push(real_type);
    }
    typed_fields
}
