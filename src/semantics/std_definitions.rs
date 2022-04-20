use std::collections::HashMap;

use crate::types::Type;

const VOID_TYPE: Type = Type::Tuple(vec![]);

pub fn raw_std_functions() -> HashMap<&'static str, (Vec<Type>, Type)> {
    HashMap::from([
        ("print", (vec![Type::String], VOID_TYPE.clone())),
        ("println", (vec![Type::String], VOID_TYPE.clone())),
        (
            "range",
            (vec![Type::Int, Type::Int], Type::List(Box::new(Type::Int))),
        ),
        ("gen_input", (vec![], Type::String)),
    ])
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
