use crate::types::Type;


pub fn get_type_from_tuple<'a>(t: &'a Type, i: usize) -> &'a Type {
    match t {
        Type::Tuple(items) => &items[i],
        _ => panic!("something is wrong, semantics should have checked this.."),
    }
}

pub fn get_type_size(t: &Type) -> u8 {
    match t {
        Type::Int => 1,
        Type::Float => 1,
        Type::Bool => 1,
        Type::String => 1,
        Type::Maybe(inner) => get_type_size(inner.as_ref()) + 1,
        Type::Tuple(items) => items.iter().map(|t| get_type_size(t)).sum(),
        Type::List(_) => 1,
        Type::Ident(_) => 1,
    }
}