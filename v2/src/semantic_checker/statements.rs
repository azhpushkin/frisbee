use std::collections::HashMap;

use crate::ast::*;

fn check_statements_in_scope(
    statements: &Vec<Statement>,
    scope: Option<ObjectDecl>,
    file: &FileAst,
) {
}
// TODO: check assignment to expression
pub fn check_statements(ast: &FileAst) {
    let first_key = ast.types.keys().next().unwrap();
    let objtype = ast.types.get(first_key).unwrap();

    let state: HashMap<String, &Type> = HashMap::new();
}
