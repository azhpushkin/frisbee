use std::fmt::{Display, Error, Formatter};

use crate::loader::ModuleAlias;
use crate::types::Type;

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

    pub fn new_std_method(t: &Type, name: &str) -> Self {
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
}
impl Into<String> for &SymbolFunc {
    fn into(self) -> String {
        self.0.clone()
    }
}

impl Display for SymbolFunc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{}", self.0)
    }
}

impl Display for SymbolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
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

    pub fn method(&self, method: &String) -> SymbolFunc {
        SymbolFunc(format!("{}::{}", self.0, method))
    }
}

impl Into<Type> for &SymbolType {
    fn into(self) -> Type {
        Type::Ident(self.0.clone())
    }
}
impl From<&Type> for SymbolType {
    fn from(t: &Type) -> Self {
        if let Type::Ident(name) = t {
            // check that ident is a correct SymbolType
            if !name.contains("::") {
                panic!("{} is not a valid SymbolType", name);
            }
            let (module, typename) = name.rsplit_once("::").unwrap();
            assert!(!module.contains("::"), "Bad module name: {}", module);
            assert!(
                typename.chars().next().unwrap().is_ascii_uppercase(),
                "Bad SymbolType: {}",
                name
            );
            SymbolType(name.clone())
        } else {
            panic!("Type {} can't be coersed to SymbolType!", t);
        }
    }
}

