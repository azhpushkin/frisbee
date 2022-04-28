use super::constants::Constant;
use super::generator::BytecodeGenerator;
use super::utils::{extract_custom_type, get_tuple_offset, get_type_from_tuple, get_type_size};
use crate::verified_ast::{RawOperator, VExpr, VExprTyped};
use crate::symbols::SymbolFunc;
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

pub fn match_std_function(symbol: &SymbolFunc) -> u8 {
    let matched_std_function = STD_RAW_FUNCTION_RUNNERS
        .iter()
        .enumerate()
        .find(|(_, (name, _))| symbol.is_eq_to_str(*name));
    match matched_std_function {
        Some((index, (_, _))) => index as u8,
        None => panic!("No std function {} found", symbol),
    }
}

impl<'a, 'b> BytecodeGenerator<'a, 'b> {
    pub fn push_expr(&mut self, expr: &VExprTyped) {
        let VExprTyped { expr, .. } = expr;
        match expr {
            VExpr::Int(i) => {
                if 0 <= *i && *i < 256 {
                    // TODO: check how 255 and 0 and -255 is handled
                    self.push(op::LOAD_SMALL_INT);
                    self.push(*i as u8);
                } else {
                    self.push(op::LOAD_CONST);
                    self.push_constant(Constant::Int(*i as i64));
                }
            }
            VExpr::Float(f) => {
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::Float(*f as f64));
            }
            VExpr::String(s) => {
                self.push(op::LOAD_CONST);
                self.push_constant(Constant::String(s.clone()));
            }
            VExpr::Bool(b) if *b => self.push(op::LOAD_TRUE),
            VExpr::Bool(_) => self.push(op::LOAD_FALSE),

            VExpr::ApplyOp { operator, operands } => {
                for operand in operands.iter() {
                    self.push_expr(operand);
                }
                self.push(match_operator(operator));
            }
            VExpr::GetVar(varname) => {
                self.push_get_local(varname);
            }
            VExpr::CallFunction { name, return_type, args } => {
                // TODO: review this, as args_num now can have variable length
                self.push_reserve(return_type);
                for arg in args.iter() {
                    self.push_expr(arg);
                }
                let func_locals_size: u8 =
                    args.iter().map(|arg| get_type_size(&arg.expr_type)).sum();

                if name.is_std() {
                    self.push(op::CALL_STD);
                    self.push_type_size(return_type);
                    self.push(func_locals_size);
                    self.push(0);
                    self.push(match_std_function(name));
                } else {
                    self.push(op::CALL);
                    self.push_type_size(return_type);
                    self.push(func_locals_size);
                    self.push_function_placeholder(name);
                }
            }
            VExpr::TupleValue(items) => {
                for item in items.iter() {
                    self.push_expr(item);
                }
            }
            VExpr::ListValue { item_type, items } => {
                for item in items.iter() {
                    self.push_expr(item);
                }
                self.push(op::ALLOCATE_LIST);
                self.push_type_size(item_type);
                self.push(items.len() as u8);
            }
            VExpr::AccessTupleItem { tuple, index } => {
                let tuple_type = &tuple.as_ref().expr_type;
                let item_type = get_type_from_tuple(tuple_type, *index);
                self.push_reserve(item_type);
                self.push_expr(tuple.as_ref());

                let offset = get_tuple_offset(tuple_type, &[*index]);
                self.push(op::GET_TUPLE_ITEM);
                self.push_type_size(tuple_type);
                self.push(offset);
                self.push_type_size(item_type);
            }
            VExpr::AccessField { object, field } => {
                let object_type = extract_custom_type(&object.expr_type);
                self.push_expr(object);
                self.push(op::GET_OBJ_FIELD);
                self.push(self.types_meta.get(object_type).field_offsets[field]);
                self.push(self.types_meta.get(object_type).field_sizes[field]);
            }
            VExpr::AccessListItem { list, index } => {
                self.push_expr(index);
                self.push_expr(list);
                self.push(op::GET_LIST_ITEM);
            }
            VExpr::Allocate { typename } => {
                self.push(op::ALLOCATE);
                self.push(self.types_meta.get(typename).size);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::stdlib;
    use crate::symbols::SymbolFunc;
    use crate::types::{Type, VerifiedType};
    use crate::vm::stdlib_runners::STD_RAW_FUNCTION_RUNNERS;

    #[test]
    fn check_that_all_std_functions_are_there() {
        let mut std_symbols: Vec<SymbolFunc> = vec![];

        std_symbols.extend(stdlib::STD_FUNCTIONS.map(|(s, _)| SymbolFunc::new_std_function(s)));
        let method_pairs: [(_, VerifiedType); 5] = [
            (stdlib::STD_BOOL_METHODS.iter(), Type::Bool),
            (stdlib::STD_INT_METHODS.iter(), Type::Int),
            (stdlib::STD_FLOAT_METHODS.iter(), Type::Float),
            (stdlib::STD_STRING_METHODS.iter(), Type::String),
            (
                stdlib::STD_LIST_METHODS.iter(),
                Type::List(Box::new(Type::Int)),
            ), // inner type does not matter
        ];
        for (methods, t) in method_pairs {
            std_symbols.extend(methods.map(|(s, _)| SymbolFunc::new_std_method(&t, s)));
        }

        let mut std_runner_names: Vec<String> = STD_RAW_FUNCTION_RUNNERS
            .iter()
            .map(|(s, _)| String::from(*s))
            .collect();

        std_runner_names.sort();
        std_symbols.sort();

        for (runner_name, symbol) in std_runner_names.iter().zip(std_symbols.iter()) {
            assert!(symbol.is_eq_to_str(runner_name));
        }
    }
}
