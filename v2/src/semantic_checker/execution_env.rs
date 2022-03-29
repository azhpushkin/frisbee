use std::collections::HashMap;

use crate::ast::*;
use crate::loader::{LoadedFile, WholeProgram};

pub type TypesMap = HashMap<(String, ModulePathAlias), ObjectDecl>;

pub struct ExecutionEnv {
    pub variables_types: HashMap<String, Type>,
    pub types_definitions: TypesMap,
    pub funcs_definitions: HashMap<String, FunctionDecl>,
    pub scope: Option<ObjectDecl>,
}

pub fn get_env_for_file(wp: &WholeProgram, file: &LoadedFile) -> ExecutionEnv {
    let mut type_definitions: HashMap<(String, ModulePathAlias), ObjectDecl> = HashMap::new();
    for (module_path, file) in wp.files.iter() {
        for (_, objdecl) in file.ast.types.iter() {
            // TODO: objdecl.clone() looks bad, need to rewrite it I believe
            type_definitions.insert((objdecl.name.clone(), module_path.clone()), objdecl.clone());
        }
    }

    let mut env = ExecutionEnv {
        variables_types: HashMap::new(),
        types_definitions: type_definitions,
        funcs_definitions: HashMap::new(),
        scope: None,
    };

    // TODO: this clone looks bad :(
    env.funcs_definitions = file.ast.functions.clone();
    for import in &file.ast.imports {
        let import_path = import.module_path.alias();
        let imported_file = wp.files.get(&import_path).unwrap();

        for func_name in &import.functions {
            let x = imported_file.ast.functions.get(func_name).unwrap();
            env.funcs_definitions.insert(x.name.clone(), x.clone());
        }
    }
    env
}
