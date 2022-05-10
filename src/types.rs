use std::fmt;

use crate::symbols::SymbolType;

/// Type of a value in a program
/// Generalized over the type of Custom (user-defined) type
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Type<T> {
    // Primitive types
    Int,
    Float,
    Bool,
    String,

    // Type wrappers
    List(Box<Type<T>>),
    Tuple(Vec<Type<T>>),
    Maybe(Box<Type<T>>),

    // User-defined type
    Custom(T),
}

pub type ParsedType = Type<String>;
pub type VerifiedType = Type<SymbolType>;

impl<T> fmt::Display for Type<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::Bool => write!(f, "Bool"),
            Self::String => write!(f, "String"),

            Self::Custom(ident) => write!(f, "{}", ident),

            Self::List(item) => write!(f, "[{}]", item),
            Self::Tuple(items) => {
                let items_str: Vec<_> = items.iter().map(|i| format!("{}", i)).collect();
                write!(f, "({})", items_str.join(", "))
            }
            Self::Maybe(inner) => write!(f, "{}?", inner),
        }
    }
}

pub fn verify_parsed_type<R, M>(source_type: &ParsedType, mapper: &M) -> Result<VerifiedType, R>
where
    M: Fn(&str) -> Result<SymbolType, R>,
{
    Ok(match source_type {
        Type::Int => Type::Int,
        Type::Float => Type::Float,
        Type::Bool => Type::Bool,
        Type::String => Type::String,

        Type::List(inner) => {
            let real_inner = verify_parsed_type(inner, mapper)?;
            Type::List(Box::new(real_inner))
        }
        Type::Tuple(items) => {
            let real_items: Result<Vec<VerifiedType>, R> =
                items.iter().map(move |t| verify_parsed_type(t, mapper)).collect();
            Type::Tuple(real_items?)
        }
        Type::Maybe(inner) => {
            let real_inner = verify_parsed_type(inner, mapper)?;
            Type::Maybe(Box::new(real_inner))
        }
        Type::Custom(ident) => Type::Custom(mapper(ident)?),
    })
}
