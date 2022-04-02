use std::collections::HashMap;

use crate::ast::*;

// These are applicable for both Types and functions
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SymbolOrigin {
    pub module: ModulePathAlias,
    pub name: String,
}

#[derive(PartialEq, Debug)]
pub struct ClassSignature {
    pub module_path_alias: ModulePathAlias,
    pub name: String,
    pub is_active: bool,
    pub fields: HashMap<String, Type>,
    pub methods: HashMap<String, FunctionSignature>,
}

#[derive(PartialEq, Debug)]
pub struct FunctionSignature {
    pub rettype: Type,
    pub args: Vec<(String, Type)>,
}

pub type SymbolOriginsMapping = HashMap<String, SymbolOrigin>;

pub struct SymbolOriginsPerFile {
    pub typenames: SymbolOriginsMapping,
    pub functions: SymbolOriginsMapping,
}

pub struct GlobalSignatures {
    pub typenames: HashMap<SymbolOrigin, ClassSignature>,
    pub functions: HashMap<SymbolOrigin, FunctionSignature>,
}

pub struct GlobalSymbolsInfo {
    pub symbols_per_file: HashMap<ModulePathAlias, SymbolOriginsPerFile>,
    pub global_signatures: GlobalSignatures,
}
