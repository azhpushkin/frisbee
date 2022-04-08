use std::collections::HashMap;

use crate::ast::*;
use crate::vm::Op;

use super::expressions::generate_expr_bytecode;

pub fn generate_function_bytecode(func: FunctionDecl) -> Vec<u8> {
    let mut function_bytecode: Vec<u8> = vec![];

    let mut locals: HashMap<&String, usize> =
        func.args.iter().enumerate().map(|(i, arg)| (&arg.name, i)).collect();

    for statement in func.statements {
        let bytecode: Vec<u8> = match statement {
            Statement::Expr(expr) => {
                let expr_bytecode = generate_expr_bytecode(&expr, &locals);
                expr_bytecode.push(Op::POP);
                expr_bytecode
            },
            _ => todo!(),
        }
        
        function_bytecode.extend(bytecode.iter());
    }
    function_bytecode
}
