use std::collections::HashMap;

pub struct CustomType {
    pub name: String,
    pub fields: TypedFields,
}

pub enum TrueType<'a> {
    Int,
    Float,
    String,
    Nil,
    Bool,

    List(Box<Self>),
    Tuple(Box<Self>),
    Custom(&'a CustomType)
}

pub struct TypedFields<'a> {
    pub field_names: HashMap<String, usize>,
    pub field_types: Vec<TrueType<'a>>
}