use crate::ast::parsed::{ClassDecl, Expr, ExprWithPos, FunctionDecl, Statement, StatementWithPos};
use crate::types::Type;

pub fn add_default_constructor(class: &mut ClassDecl) {
    if class.methods.iter().any(|x| x.name == class.name) {
        // Constructor already exist, move on
        return;
    }

    let pos = class.pos;
    let mut statements: Vec<Statement> = vec![];
    let dummy_expr_with_pos = |expr| ExprWithPos { expr, pos_first: pos, pos_last: pos };
    for field in class.fields.iter() {
        // TODO: review ExprWithPos usage here
        let left = Expr::OwnFieldAccess { field: field.name.clone() };
        let right = Expr::Identifier(field.name.clone());
        statements.push(Statement::Assign {
            left: dummy_expr_with_pos(left),
            right: dummy_expr_with_pos(right),
        });
    }
    let default_constructor = FunctionDecl {
        pos: class.pos,
        name: class.name.clone(),
        rettype: Some(Type::Custom(class.name.clone())),
        args: class.fields.clone(),
        statements: statements
            .into_iter()
            .map(|s| StatementWithPos { statement: s, pos })
            .collect(),
    };
    class.methods.push(default_constructor);
}
