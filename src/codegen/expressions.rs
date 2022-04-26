use std::convert::TryFrom;

use super::constants::Constant;
use super::generator::BytecodeGenerator;
use super::utils::{get_tuple_offset, get_type_from_tuple};
use crate::semantics::light_ast::{LExpr, LExprTyped, RawOperator};
use crate::semantics::symbols::SymbolFunc;
use crate::types::Type;
use crate::vm::opcodes::op;
use crate::vm::stdlib_runners::STD_RAW_FUNCTION_RUNNERS;

fn match_operator(raw_op: &RawOperator) -> u8 {
    match raw_op {
        RawOperator::UnaryNegateInt => op::NEGATE_INT,
        RawOperator::AddInts => op::ADD_INT,
        RawOperator::SubInts => op::SUB_INT,
        RawOperator::MulInts => op::MUL_INT,
        RawOperator::DivInts => op::DIV_INT,
        RawOperator::GreaterInts => op::GREATER_INT,
        RawOperator::LessInts => op::LESS_INT,
        RawOperator::EqualInts => op::EQ_INT,

        RawOperator::UnaryNegateFloat => op::NEGATE_FLOAT,
        RawOperator::AddFloats => op::ADD_FLOAT,
        RawOperator::SubFloats => op::SUB_FLOAT,
        RawOperator::MulFloats => op::MUL_FLOAT,
        RawOperator::DivFloats => op::DIV_FLOAT,
        RawOperator::GreaterFloats => op::GREATER_FLOAT,
        RawOperator::LessFloats => op::LESS_FLOAT,
        RawOperator::EqualFloats => op::EQ_FLOAT,

        RawOperator::UnaryNegateBool => op::NEGATE_BOOL,
        RawOperator::EqualBools => op::EQ_BOOL,
        RawOperator::AndBools => op::AND_BOOL,
        RawOperator::OrBools => op::OR_BOOL,

        RawOperator::EqualStrings => op::EQ_STRINGS,
        RawOperator::AddStrings => op::ADD_STRINGS,
    }
}

pub fn match_std_function(name: &SymbolFunc) -> u8 {
    let name_s: String = name.into();
    let matched_std_function = STD_RAW_FUNCTION_RUNNERS
        .iter()
        .enumerate()
        .find(|(_, (name, _))| *name == name_s.as_str());
    match matched_std_function {
        Some((index, (_, _))) => index as u8,
        None => panic!("No std function {} found", name_s),
    }
}

impl<'a, 'b> BytecodeGenerator<'a, 'b> {
    pub fn push_expr(&mut self, expr: &LExprTyped) {
        let LExprTyped { expr, expr_type } = expr;
        match expr {
            LExpr::Int(i) => {
                if 0 <= *i && *i < 256 {
                    // TODO: check how 255 and 0 and -255 is handled
                    self.push(op::LOAD_SMALL_INT);
                    self.push(*i as u8);
                } else {
                    self.push(op::LOAD_CONST);
                    self.push_constant(Constant::Int(*i as i64));
                }
            }
            LExpr::Float(f) => {
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::Float(*f as f64));
            }
            LExpr::String(s) => {
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::String(s.clone()));
            }
            LExpr::Bool(b) if *b => self.push(op::LOAD_TRUE),
            LExpr::Bool(_) => self.push(op::LOAD_FALSE),

            LExpr::ApplyOp { operator, operands } => {
                for operand in operands.iter() {
                    self.push_expr(&operand);
                }
                self.push(match_operator(operator));
            }
            LExpr::GetVar(varname) => {
                self.push_get_local(varname);
            }
            LExpr::CallFunction { name, return_type, args } => {
                // TODO: review this, as args_num now can have variable length
                self.push_reserve(return_type);
                for arg in args.iter() {
                    self.push_expr(&arg);
                }
                let func_locals_size: u8 =
                    args.iter().map(|arg| arg.expr_type.get_size()).sum();

                if name.is_std() {
                    self.push(op::CALL_STD);
                    self.push(return_type.get_size());
                    self.push(func_locals_size);
                    self.push(0);
                    self.push(match_std_function(name));
                } else {
                    self.push(op::CALL);
                    self.push(return_type.get_size());
                    self.push(func_locals_size);
                    self.push_function_placeholder(name);
                }
            }
            LExpr::TupleValue(items) => {
                for item in items.iter() {
                    self.push_expr(&item);
                }
            }
            LExpr::ListValue { item_type, items } => {
                for item in items.iter() {
                    self.push_expr(&item);
                }
                self.push(op::ALLOCATE_LIST);
                self.push(item_type.get_size());
                self.push(items.len() as u8);
            },
            LExpr::AccessTupleItem { tuple, index } => {
                let tuple_type = &tuple.as_ref().expr_type;
                let item_type = get_type_from_tuple(tuple_type, *index);
                self.push_reserve(item_type);
                self.push_expr(tuple.as_ref());

                let offset = get_tuple_offset(tuple_type, &[*index]);
                self.push(op::GET_TUPLE_ITEM);
                self.push(tuple_type.get_size());
                self.push(offset);
                self.push(item_type.get_size());
            }
            LExpr::AccessField { object, field } => {
                let object_type = object.expr_type.clone().into();
                self.push_expr(&object);
                self.push(op::GET_FROM_HEAP);
                self.push(self.types_meta.get(&object_type).field_offsets[field]);
                self.push(self.types_meta.get(&object_type).field_sizes[field]);
            }
            LExpr::Allocate { typename } => {
                self.push(op::ALLOCATE);
                self.push(self.types_meta.get(typename).size);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::semantics::symbols::SymbolFunc;
    use crate::stdlib;
    use crate::types::Type;
    use crate::vm::stdlib_runners::STD_RAW_FUNCTION_RUNNERS;

    #[test]
    fn check_that_all_std_functions_are_there() {
        let mut std_symbols: Vec<SymbolFunc> = vec![];

        std_symbols.extend(stdlib::STD_FUNCTIONS.map(|(s, _)| SymbolFunc::new_std_function(s)));
        let method_pairs = [
            (stdlib::STD_BOOL_METHODS.iter(), Type::Bool),
            (stdlib::STD_INT_METHODS.iter(), Type::Int),
            (stdlib::STD_FLOAT_METHODS.iter(), Type::Float),
            (stdlib::STD_STRING_METHODS.iter(), Type::String),
        ];
        for (methods, t) in method_pairs {
            std_symbols.extend(methods.map(|(s, _)| SymbolFunc::new_std_method(&t, s)));
        }

        let mut implemented_std_functions: Vec<String> = STD_RAW_FUNCTION_RUNNERS
            .iter()
            .map(|(s, _)| String::from(*s))
            .collect();
        let mut typechecked_std_functions: Vec<String> =
            std_symbols.iter().map(|s| s.into()).collect();

        implemented_std_functions.sort();
        typechecked_std_functions.sort();

        assert_eq!(implemented_std_functions, typechecked_std_functions);
    }
}
