use std::fmt;

use crate::alias::ModuleAlias;
use crate::types::Type;

pub static MAIN_FUNCTION_NAME: &str = "main";

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct SymbolType(String);

#[derive(Debug, PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolFunc(String);

impl SymbolFunc {
    pub fn new<S>(alias: &ModuleAlias, name: S) -> Self
    where
        S: Into<String>,
    {
        Self(format!("{}::{}", alias, name.into()))
    }

    pub fn new_std_function(name: &str) -> Self {
        Self(format!("std::{}", name))
    }

    pub fn new_std_method<T: fmt::Display>(t: &Type<T>, name: &str) -> Self {
        match t {
            Type::Int => Self(format!("std::Int::{}", name)),
            Type::Float => Self(format!("std::Float::{}", name)),
            Type::Bool => Self(format!("std::Bool::{}", name)),
            Type::String => Self(format!("std::String::{}", name)),
            Type::List(..) => Self(format!("std::List::{}", name)),
            _ => panic!("Cant create std method {} for {} type", name, t),
        }
    }

    pub fn is_std(&self) -> bool {
        self.0.starts_with("std::")
    }

    pub fn is_eq_to_str(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl fmt::Display for SymbolFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl SymbolType {
    pub fn new<S>(alias: &ModuleAlias, name: S) -> Self
    where
        S: Into<String>,
    {
        Self(format!("{}::{}", alias, name.into()))
    }

    pub fn method(&self, method: &str) -> SymbolFunc {
        SymbolFunc(format!("{}::{}", self.0, method))
    }

    pub fn constructor(&self) -> SymbolFunc {
        let (_, name) = self
            .0
            .split_once("::")
            .unwrap_or_else(|| panic!("No :: in SymbolType {}", self));
        println!(
            "Constructor for {} is {}",
            self,
            SymbolFunc(format!("{}::{}", self.0, name))
        );
        SymbolFunc(format!("{}::{}", self.0, name))
    }
}
