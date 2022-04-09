use std::collections::HashMap;

use super::generator::BytecodeGenerator;
use super::globals::*;
use crate::ast::*;
use crate::vm::Op;

macro_rules! accept_typed_expr {
    ($self:ident) => {
        match $self {
            Expr::TypedExpr { expr, typename } => (expr, typename),
            _ => {
                panic!("Not typed expression, got {:?}!", $self);
            }
        }
    };
}

impl<'a> BytecodeGenerator<'a> {
    pub fn push_expr(&mut self, expr: &Expr) {
        let (inner_expr, typename) = accept_typed_expr!(expr);
        match inner_expr.as_ref() {
            Expr::Int(i) => {
                self.push(Op::LOAD_CONST);
                self.push_constant(Constant::Int(*i as i64));
            }
            Expr::Float(f) => {
                self.push(Op::LOAD_CONST);
                self.push_constant(Constant::Float(*f as f64));
            }
            Expr::BinOp { left, right, op } => {
                self.push_expr(left.as_ref());
                self.push_expr(right.as_ref());
                match (typename, op) {
                    (Type::Int, BinaryOp::Plus) => self.push(Op::ADD_INT),
                    (Type::Int, BinaryOp::Minus) => self.push(Op::SUB_INT),
                    (Type::Int, BinaryOp::Multiply) => self.push(Op::MUL_INT),
                    (Type::Int, BinaryOp::Divide) => self.push(Op::DIV_INT),

                    (Type::Float, BinaryOp::Plus) => self.push(Op::ADD_FLOAT),
                    (Type::Float, BinaryOp::Minus) => self.push(Op::SUB_FLOAT),
                    (Type::Float, BinaryOp::Multiply) => self.push(Op::MUL_FLOAT),
                    (Type::Float, BinaryOp::Divide) => self.push(Op::DIV_FLOAT),

                    _ => panic!("Sorry, no support for {:?} and {:?} now ", typename, op),
                }
            }
            Expr::Identifier(varname) => {
                self.push_get_var(varname);
            }
            // /////// Expr::FunctionCall { function: (), args: () }
            // TODO: function call to qualified function call
            // TODO: method call to qualified function call
            f => panic!("Not done yet {:?}", f),
        }
    }
}
