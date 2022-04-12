use crate::ast::{Type, ModulePathAlias};


#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct SymbolType(String);

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct SymbolFunc(String);

pub fn compile_typename(alias: &ModulePathAlias, name: &String) -> SymbolType {
    SymbolType(format!("{}::{}", alias.0, name))
}

pub fn compile_func(alias: &ModulePathAlias, name: &String) -> SymbolFunc {
    SymbolFunc(format!("{}::{}", alias.0, name))
}
pub fn compile_method(alias: &ModulePathAlias, typename: &String, method: &String) -> SymbolFunc {
    SymbolFunc(format!("{}::{}::{}", alias.0, typename, method))
}

impl Into<Type> for SymbolType {
    fn into(self) -> Type {
        Type::Ident(self.0.clone())
    }
}
impl From<&Type> for SymbolType {
    fn from(t: &Type) -> Self {
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
