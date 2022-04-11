use crate::ast::{ClassDecl, Expr, FunctionDecl, Statement, Type};


pub fn add_default_constructor(class: &mut ClassDecl) {
    if class.methods.iter().find(|x| x.name == class.name).is_some() {
        // Constructor already exist, move on
        return;
    }

    let mut statements: Vec<Statement> = vec![];
    for field in class.fields.iter() {
        statements.push(Statement::Assign {
            left: Expr::OwnFieldAccess { field: field.name.clone() },
            right: Expr::Identifier(field.name.clone()),
        });
    }
    let default_constructor = FunctionDecl {
        name: class.name.clone(),
        rettype: Some(Type::Ident(class.name.clone())),
        args: class.fields.clone(),
        statements,
    };
}
