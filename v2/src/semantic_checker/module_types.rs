use crate::ast::*;
use crate::loader::{LoadedFile, WholeProgram};
use std::collections::HashMap;

// pub fn check_module_types(wp: &WholeProgram) {
//     for (_, file) in &wp.files {
//         let functions: HashMap<String, &FunctionDecl> = file.ast.functions.iter().map(|x| (x.name.clone(), x)).collect();
//         let actives: HashMap<String, &ObjectDecl> = file.ast.actives.iter().map(|x| (x.name.clone(), x)).collect();
//         let classes: HashMap<String, &ObjectDecl> = file.ast.classes.iter().map(|x| (x.name.clone(), x)).collect();
//     }
// }
