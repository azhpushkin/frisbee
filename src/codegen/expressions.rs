use std::collections::HashMap;

use crate::vm::Op;
use crate::ast::*;
use super::globals::*;

macro_rules! accept_typed_expr {
    ($self:ident) => {
        match $self {
            Expr::TypedExpr{expr, typename} => (expr, typename),
            _ => { return Err(format!("Not typed expression, got {:?}!", $self)); }
        }
    };
}



pub struct ExprBytecodeGenerator<'a> {
    globals: &'a Globals,
    locals: &'a HashMap<&'a String, usize>,
    pub bytecode: Vec<u8>
}

impl<'a> ExprBytecodeGenerator<'a> {
    pub fn new(globals: &'a Globals, locals: &'a HashMap<&'a String, usize>) -> Self {
        ExprBytecodeGenerator {
            globals,
            locals,
            bytecode: vec![],
        }
    }
    
    pub fn generate(&mut self, expr: &Expr) -> Result<(), String> {
        let (inner_expr, typename) = accept_typed_expr!(expr);
        match expr {
            Expr::BinOp { left, right, op } => {
                self.generate(left.as_ref());
                self.generate(right.as_ref());
                match (typename, op) {
                    (Type::Int, BinaryOp::Plus) => self.bytecode.push(Op::ADD_INT),
                    (Type::Int, BinaryOp::Minus) => self.bytecode.push(Op::SUB_INT),
                    (Type::Int, BinaryOp::Multiply) => self.bytecode.push(Op::MUL_INT),
                    (Type::Int, BinaryOp::Divide) => self.bytecode.push(Op::DIV_INT),

                    (Type::Float, BinaryOp::Plus) => self.bytecode.push(Op::ADD_FLOAT),
                    (Type::Float, BinaryOp::Minus) => self.bytecode.push(Op::SUB_FLOAT),
                    (Type::Float, BinaryOp::Multiply) => self.bytecode.push(Op::MUL_FLOAT),
                    (Type::Float, BinaryOp::Divide) => self.bytecode.push(Op::DIV_FLOAT),

                    _ => panic!("Sorry, no support for {:?} and {:?} now ", typename, op)
                }
            }
            _ => todo!(),
        }
        Ok(())
    }

}


