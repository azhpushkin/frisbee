use crate::types::Type;
use crate::loader::ModuleAlias;

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

    pub fn is_std(&self) -> bool {
        self.0.starts_with("std::")
    }
}
impl Into<String> for &SymbolFunc {
    fn into(self) -> String {
        self.0.clone()
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
impl From<Type> for SymbolType {
    fn from(t: Type) -> Self {
        if let Type::Ident(name) = t {
            // check that ident is a correct SymbolType
            let (module, typename) = name.rsplit_once("::").expect("Not a SymbolType");
            assert!(module.find("::").is_none(), "Bad module name: {}", module);
            assert!(
                typename.chars().next().unwrap().is_ascii_uppercase(),
                "Bad SymbolType: {}",
                name
            );
            SymbolType(name.clone())
        } else {
            panic!("Expected TypeIdent, got {:?}", t);
        }
    }
}
