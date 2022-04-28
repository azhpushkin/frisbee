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
    // IdentQualified(SymbolType)
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

impl Type {
    pub fn get_size(&self) -> u8 {
        match self {
            Type::Int => 1,
            Type::Float => 1,
            Type::Bool => 1,
            Type::String => 1,
            Type::Maybe(inner) => inner.as_ref().get_size() + 1,
            Type::Tuple(items) => items.iter().map(|t| t.get_size()).sum(),
            Type::List(_) => 1,
            Type::Ident(_) => 1,
        }
    }
}
