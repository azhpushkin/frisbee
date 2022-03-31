use std::collections::HashMap;

use super::modules::{GlobalSignatures, SymbolOrigin, SymbolOriginsPerFile};
use crate::ast::*;

pub struct TypeEnv {
    pub variables_types: HashMap<String, Type>,
    pub symbol_origins: SymbolOriginsPerFile,
    pub signatures: GlobalSignatures,
    pub scope: Option<SymbolOrigin>,
}
