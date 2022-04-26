use std::fmt;

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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::Bool => write!(f, "Bool"),
            Self::String => write!(f, "String"),

            Self::Ident(ident) => write!(f, "{}", ident),

            Self::List(item) => write!(f, "[{}]", item),
            Self::Tuple(items) => {
                let items_str: Vec<_> = items.iter().map(|i| format!("{}", i)).collect();
                write!(f, "({})", items_str.join(", "))
            }
            Self::Maybe(inner) => write!(f, "{}?", inner),
        }
    }
}
