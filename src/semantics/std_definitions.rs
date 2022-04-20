use std::collections::HashMap;

use crate::loader::{generate_alias, ModuleAlias};
use crate::types::Type;

use super::aggregate::RawFunction;
use super::annotations::TypedFields;
use super::symbols::SymbolFunc;

fn std_symbol_func(name: &str) -> SymbolFunc {
    let std_module = generate_alias(&vec!["std".into()]);
    SymbolFunc::new(&std_module, name)
}

fn std_function_signatures() -> HashMap<&'static str, (Vec<Type>, Type)> {
    HashMap::from([
        ("print", (vec![Type::String], Type::Tuple(vec![]))),
        ("println", (vec![Type::String], Type::Tuple(vec![]))),
        (
            "range",
            (vec![Type::Int, Type::Int], Type::List(Box::new(Type::Int))),
        ),
        ("input", (vec![], Type::String)),
    ])
}

pub fn get_std_raw_signature(name: &String) -> RawFunction {
    let (args, return_type) = &std_function_signatures()[name.as_str()];
    RawFunction {
        name: std_symbol_func(name.as_str()),
        return_type: Some(return_type.clone()),
        args: TypedFields {
            types: args.clone(),
            names: args.iter().enumerate().map(|(i, _)| (i, "".into())).collect(),
        },
        body: vec![],
        short_name: name.clone(),
        method_of: None,
        defined_at: generate_alias(&vec!["std".into()]),
    }
}

pub const STD_FUNCTION_NAMES: [&str; 4] = ["print", "println", "range", "get_input"];

// print(String) -> void
// println(String) -> void
// range(Int, Int) -> [Int]

// Bool
// to_string

// Int
// * to_float
// * to_string
// * abs

// Float
// * to_string
// * abs
// * ceil
// * floor
// * round

// String:
// * len
// * is_empty  -> shortcut for len == 0
// * contains -> shortcut to find != nil
// * find -> Int?

// List
// * push
// * pop
// * len
// * is_empty
