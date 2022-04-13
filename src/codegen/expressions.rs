use super::constants::Constant;
use super::generator::BytecodeGenerator;
use crate::ast::Type;
use crate::semantics::light_ast::{LExprTyped, LExpr, RawOperator};
use crate::semantics::symbols::SymbolFunc;
use crate::vm::opcodes::op;


fn match_operator(raw_op: &RawOperator) -> u8 {
    match raw_op {
        RawOperator::UnaryNegateInt => todo!(),
        RawOperator::AddInts => op::ADD_INT,
        RawOperator::SubInts => op::SUB_INT,
        RawOperator::MulInts => op::MUL_INT,
        RawOperator::DivInts => op::DIV_INT,
        RawOperator::GreaterInts => todo!(),
        RawOperator::LessInts => todo!(),
        RawOperator::EqualInts => todo!(),
        
        RawOperator::UnaryNegateFloat => todo!(),
        RawOperator::AddFloats => op::ADD_FLOAT,
        RawOperator::SubFloats => op::SUB_FLOAT,
        RawOperator::MulFloats => op::MUL_FLOAT,
        RawOperator::DivFloats => op::DIV_FLOAT,
        RawOperator::GreaterFloats => todo!(),
        RawOperator::LessFloats => todo!(),
        RawOperator::EqualFloats => todo!(),

        RawOperator::UnaryNegateBool => todo!(),
    }
    
}


impl<'a, 'b> BytecodeGenerator<'a, 'b> {
    pub fn push_expr(&mut self, expr: &LExprTyped) {
        let LExprTyped {expr, expr_type} = expr;
        match expr {
            LExpr::Int(i) => {
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::Int(*i as i64));
            }
            LExpr::Float(f) => {
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::Float(*f as f64));
            }
            LExpr::String(_) => todo!("load string is not done!"),
            LExpr::Bool(b) if *b => self.push(op::LOAD_TRUE),
            LExpr::Bool(_) => self.push(op::LOAD_FALSE),
                
            LExpr::ApplyOp { operator, operands } => {
                for operand in operands.iter() {
                    self.push_expr(&operand);   
                }
                self.push(match_operator(operator));
            }
            LExpr::GetVar(varname) => {
                self.push_get_var(varname);
            }
            LExpr::CallFunction{name, args} => {
                for arg in args.iter() {
                    self.push_expr(&arg);
                }
                self.push(op::CALL);
                self.push(args.len() as u8);

                
            }
            LExpr::Allocate { .. } => todo!("Allocate is not here yet!"),
        }
    }
}
