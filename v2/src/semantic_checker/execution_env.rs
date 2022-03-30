use std::collections::HashMap;

use crate::ast::*;

pub struct ExecutionEnv {
    pub variables_types: HashMap<String, Type>,
    pub types_definitions: HashMap<String, ObjectDecl>,
    pub funcs_definitions: HashMap<String, FunctionDecl>,
    pub scope: Option<ObjectDecl>,
}
