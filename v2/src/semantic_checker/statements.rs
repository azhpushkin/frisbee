use std::collections::HashMap;

use crate::ast::*;
use crate::loader::LoadedFile;

use super::expressions::ExprTypeChecker;
use super::modules::{GlobalSignatures, SymbolOriginsPerFile};
use super::semantic_error::{sem_err, SemanticResult};
use super::type_env::TypeEnv;

pub fn check_statements_in_file(
    file: &LoadedFile,
    symbol_origins: &SymbolOriginsPerFile,
    signatures: &GlobalSignatures,
) -> SemanticResult<()> {
    let mut env = TypeEnv {
        variables_types: HashMap::new(),
        symbol_origins: symbol_origins,
        signatures: signatures,
        scope: None,
    };
    for func in &file.ast.functions {
        let origin = symbol_origins.functions.get(&func.name).unwrap();
        let func_signature = signatures.functions.get(&origin).unwrap();
        env.variables_types = func_signature.args.clone().into_iter().collect();
        check_statements(&func.statements, &mut env)?;
    }
    Ok(())
}

// TODO: check assignment to expression
pub fn check_statements(statements: &Vec<Statement>, env: &mut TypeEnv) -> SemanticResult<()> {
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

    Ok(())
}
