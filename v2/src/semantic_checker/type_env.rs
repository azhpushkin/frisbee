use std::collections::HashMap;

use super::modules::{GlobalSignatures, SymbolOrigin, SymbolOriginsPerFile};
use crate::ast::*;

pub struct TypeEnv<'a> {
    pub variables_types: HashMap<String, Type>,
    pub symbol_origins: &'a SymbolOriginsPerFile,
    pub signatures: &'a GlobalSignatures,
    pub scope: Option<SymbolOrigin>,
}
