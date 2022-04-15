#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // Primitive types
    Int,
    Float,
    Bool,
    String,

    // Type wrappers
    List(Box<Type>),
    Tuple(Vec<Type>),
    Maybe(Box<Type>),

    // User-defined type
    Ident(String),
}