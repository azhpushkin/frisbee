use std::collections::HashMap;

use crate::ast::*;

use super::execution_env::ExecutionEnv;
use super::expressions::ExprTypeChecker;

// TODO: check assignment to expression
pub fn check_statements(statements: &Vec<Statement>, env: &mut ExecutionEnv) {
    for statement in statements.iter() {
        match statement {
            Statement::SVarDecl(t, s) => {
                env.variables_types.insert(s.clone(), t.clone());
            }
            Statement::SVarDeclEqual(t, s, e) => {
                let expr_t = ExprTypeChecker::new(env).calculate(e).expect("1111");
                if expr_t != *t {
                    panic!("Expressions not match at {:?}", statement);
                }
                env.variables_types.insert(s.clone(), t.clone());
            }
            _ => panic!("{:?} not implemented", statement),
        }
    }
}
