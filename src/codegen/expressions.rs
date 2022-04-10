use super::constants::Constant;
use super::generator::BytecodeGenerator;
use crate::ast::*;
use crate::vm::op;

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
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::Int(*i as i64));
            }
            Expr::Float(f) => {
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::Float(*f as f64));
            }
            Expr::BinOp { left, right, op } => {
                self.push_expr(left.as_ref());
                self.push_expr(right.as_ref());
                match (typename, op) {
                    (Type::Int, BinaryOp::Plus) => self.push(op::ADD_INT),
                    (Type::Int, BinaryOp::Minus) => self.push(op::SUB_INT),
                    (Type::Int, BinaryOp::Multiply) => self.push(op::MUL_INT),
                    (Type::Int, BinaryOp::Divide) => self.push(op::DIV_INT),

                    (Type::Float, BinaryOp::Plus) => self.push(op::ADD_FLOAT),
                    (Type::Float, BinaryOp::Minus) => self.push(op::SUB_FLOAT),
                    (Type::Float, BinaryOp::Multiply) => self.push(op::MUL_FLOAT),
                    (Type::Float, BinaryOp::Divide) => self.push(op::DIV_FLOAT),

                    _ => panic!("Sorry, no support for {:?} and {:?} now ", typename, op),
                }
            }
            Expr::Identifier(varname) => {
                self.push_get_var(varname);
            }
            Expr::FunctionCall { .. } => {
                panic!("FunctionCall should not be here: {:?}", inner_expr)
            }
            Expr::FunctionCallQualified { module, function, args } => {
                for arg in args.iter() {
                    self.push_expr(&arg);
                }
                let function_id = self.get_function(module, function) as u8;
                self.push(op::CALL);
                self.push(function_id);
                for _ in args.iter() {
                    self.push(op::POP);
                }
            }
            // /////// Expr::FunctionCall { function: (), args: () }
            // TODO: function call to qualified function call
            // TODO: method call to qualified function call
            f => panic!("Not done yet {:?}", f),
        }
    }
}