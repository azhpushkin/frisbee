use super::constants::Constant;
use super::generator::BytecodeGenerator;
use super::utils::{get_tuple_offset, get_tuple_subitem_type, get_type_size, unwrap_type_as};
use crate::ast::verified::{RawOperator, VExpr, VExprTyped};
use crate::runtime::opcodes::op;
use crate::runtime::stdlib_runners::STD_RAW_FUNCTION_RUNNERS;
use crate::symbols::SymbolFunc;
use crate::types::Type;

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

impl<'a> BytecodeGenerator<'a> {
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
            VExpr::TernaryOp { condition, if_true, if_false } => {
                self.push_expr(condition);

                self.push(op::JUMP_IF_FALSE);
                let placeholder_to_skip_ifbody = self.push_placeholder();

                self.push_expr(if_true);
                self.push(op::JUMP);
                let placeholder_to_skip_elsebody = self.push_placeholder();
                self.fill_placeholder(&placeholder_to_skip_ifbody);

                self.push_expr(if_false);
                self.fill_placeholder(&placeholder_to_skip_elsebody);
            }
            VExpr::GetVar(varname) => {
                let var_pos = *self.locals.get(varname.as_str()).unwrap();
                self.push(op::GET_LOCAL);
                self.push(var_pos);
                self.push_type_size(self.locals_types[varname.as_str()]);
            }
            VExpr::CallFunction { name, return_type, args } => {
                if !name.is_std() {
                    self.push_reserve(return_type);
                }
                for arg in args.iter() {
                    self.push_expr(arg);
                }
                let func_locals_size: u8 =
                    args.iter().map(|arg| get_type_size(&arg.expr_type)).sum();

                if name.is_std() {
                    self.push(op::CALL_STD);
                    self.push(func_locals_size);
                    self.push(0);
                    self.push(match_std_function(name));
                } else {
                    self.push(op::CALL);
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

                let list_flag = self.list_kinds_meta.get_or_insert(item_type);

                self.push(op::ALLOCATE_LIST);
                self.push(list_flag as u8);
                self.push(items.len() as u8);
            }
            VExpr::AccessTupleItem { tuple, index } => {
                let tuple_type = &tuple.as_ref().expr_type;
                let item_type = get_tuple_subitem_type(tuple_type, *index);
                self.push_reserve(item_type);
                self.push_expr(tuple.as_ref());

                let offset = get_tuple_offset(tuple_type, &[*index]);
                self.push(op::GET_TUPLE_ITEM);
                self.push_type_size(tuple_type);
                self.push(offset);
                self.push_type_size(item_type);
            }
            VExpr::AccessField { object, field } => {
                let object_type = unwrap_type_as!(&object.expr_type, Type::Custom);
                self.push_expr(object);
                self.push(op::GET_OBJ_FIELD);
                self.push(self.custom_types_meta.get_meta(object_type).field_offsets[field]);
                self.push(self.custom_types_meta.get_meta(object_type).field_sizes[field]);
            }
            VExpr::AccessListItem { list, index } => {
                self.push_expr(index);
                self.push_expr(list);
                self.push(op::GET_LIST_ITEM);
            }
            VExpr::Allocate { typename } => {
                self.push(op::ALLOCATE);
                self.push(self.custom_types_meta.get_index(typename) as u8);
            }
            VExpr::Spawn { typename, args } => {
                self.push(op::RESERVE);
                self.push(1);
                for arg in args {
                    self.push_expr(arg);
                }

                let constructor_name = typename.constructor();

                self.push(op::SPAWN);
                self.push(self.custom_types_meta.get_index(typename) as u8);
                self.push_function_placeholder(&constructor_name);
            }
            VExpr::Dummy(t) => {
                self.push_reserve(t);
            }
            VExpr::CurrentActive => {
                self.push(op::CURRENT_ACTIVE);
            }
            VExpr::CurrentActiveField { active_type, field } => {
                self.push(op::GET_CURRENT_ACTIVE_FIELD);
                self.push(self.custom_types_meta.get_meta(active_type).field_offsets[field]);
                self.push(self.custom_types_meta.get_meta(active_type).field_sizes[field]);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::runtime::stdlib_runners::STD_RAW_FUNCTION_RUNNERS;
    use crate::stdlib;
    use crate::symbols::SymbolFunc;
    use crate::types::{Type, VerifiedType};

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
