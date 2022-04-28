use crate::ast::TypedNamedObject;
use crate::types::Type;
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
    pub types: Vec<Type>,
    pub names: HashMap<usize, String>,
}

impl TypedFields {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Type)> {
        self.types.iter().enumerate().map(move |(i, t)| (&self.names[&i], t))
    }
}

pub fn annotate_type(
    source_type: &Type,
    custom_resolver: &SymbolResolver<SymbolType>,
) -> Result<Type, String> {
    let t = match source_type {
        Type::Int => Type::Int,
        Type::Float => Type::Float,
        Type::Bool => Type::Bool,
        Type::String => Type::String,

        Type::List(inner) => {
            let real_inner = annotate_type(inner.as_ref(), custom_resolver)?;
            Type::List(Box::new(real_inner))
        }
        Type::Tuple(items) => {
            let real_items: Result<Vec<_>, _> =
                items.iter().map(|t| annotate_type(t, custom_resolver)).collect();
            Type::Tuple(real_items?)
        }
        Type::Maybe(inner) => {
            let real_inner = annotate_type(inner.as_ref(), custom_resolver)?;
            Type::Tuple(vec![Type::Bool, real_inner])
        }
        Type::Ident(ident) => (&custom_resolver(ident)?).into(),
    };
    Ok(t)
}

pub fn annotate_typednamed_vec(
    v: &[TypedNamedObject],
    resolver: &SymbolResolver<SymbolType>,
) -> Result<TypedFields, String> {
    let mut typed_fields = TypedFields { names: HashMap::new(), types: vec![] };

    for (i, old_type) in v.iter().enumerate() {
        let real_type = annotate_type(&old_type.typename, resolver)?;

        typed_fields.names.insert(i, old_type.name.clone());
        typed_fields.types.push(real_type);
    }
    Ok(typed_fields)
}
