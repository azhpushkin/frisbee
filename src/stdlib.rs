use crate::types::Type;

pub type StdFunc = (&'static str, fn() -> (Vec<Type>, Type));

pub const STD_FUNCTIONS: [StdFunc; 4] = [
    ("print", || (vec![Type::String], Type::Int)),
    ("println", || (vec![Type::String], Type::Int)),
    (
        "range",
        || (vec![Type::Int, Type::Int], Type::List(Box::new(Type::Int))),
    ),
    ("get_input", || (vec![], Type::String)),
];

pub const STD_BOOL_METHODS: [StdFunc; 1] = [("to_string", || (vec![], Type::String))];

pub const STD_INT_METHODS: [StdFunc; 3] = [
    ("to_float", || (vec![], Type::Float)),
    ("to_string", || (vec![], Type::String)),
    ("abs", || (vec![], Type::Int)),
];

pub const STD_FLOAT_METHODS: [StdFunc; 5] = [
    ("to_string", || (vec![], Type::String)),
    ("abs", || (vec![], Type::Float)),
    ("ceil", || (vec![], Type::Int)),
    ("floor", || (vec![], Type::Int)),
    ("round", || (vec![], Type::Int)),
];

pub const STD_STRING_METHODS: [StdFunc; 4] = [
    ("len", || (vec![], Type::Int)),
    ("is_empty", || (vec![], Type::Bool)),
    (
        "find",
        || (vec![Type::String], Type::Maybe(Box::new(Type::Int))),
    ),
    ("contains", || (vec![Type::String], Type::Bool)),
];



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