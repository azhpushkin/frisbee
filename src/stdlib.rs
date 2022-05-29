use crate::types::{Type, VerifiedType};

pub type StdFunction<'a> = (&'a str, fn() -> (Vec<VerifiedType>, VerifiedType));
pub type StdMethod<'a> = (
    &'a str,
    fn(&VerifiedType) -> (Vec<VerifiedType>, VerifiedType),
);
const VOID_TYPE: VerifiedType = Type::Tuple(vec![]);

pub const STD_FUNCTIONS: [StdFunction; 6] = [
    ("print", || (vec![Type::String], VOID_TYPE)),
    ("println", || (vec![Type::String], VOID_TYPE)),
    ("fprint", || (vec![Type::String, Type::List(Box::new(Type::String))], VOID_TYPE)),
    ("fprintln", || (vec![Type::String, Type::List(Box::new(Type::String))], VOID_TYPE)),
    ("range", || {
        (vec![Type::Int, Type::Int], Type::List(Box::new(Type::Int)))
    }),
    ("get_input", || (vec![], Type::String)),
];

pub const STD_BOOL_METHODS: [StdMethod; 1] = [("to_string", |_| (vec![], Type::String))];

pub const STD_INT_METHODS: [StdMethod; 3] = [
    ("to_float", |_| (vec![], Type::Float)),
    ("to_string", |_| (vec![], Type::String)),
    ("abs", |_| (vec![], Type::Int)),
];

pub const STD_FLOAT_METHODS: [StdMethod; 5] = [
    ("to_string", |_| (vec![], Type::String)),
    ("abs", |_| (vec![], Type::Float)),
    ("ceil", |_| (vec![], Type::Int)),
    ("floor", |_| (vec![], Type::Int)),
    ("round", |_| (vec![], Type::Int)),
];

pub const STD_STRING_METHODS: [StdMethod; 4] = [
    ("len", |_| (vec![], Type::Int)),
    ("is_empty", |_| (vec![], Type::Bool)),
    ("find", |_| {
        (vec![Type::String], Type::Maybe(Box::new(Type::Int)))
    }),
    ("contains", |_| (vec![Type::String], Type::Bool)),
];

macro_rules! list_item_type {
    ($t:expr) => {
        match $t {
            Type::List(item) => item.as_ref().clone(),
            _ => panic!("expected list type"),
        }
    };
}

pub const STD_LIST_METHODS: [StdMethod; 4] = [
    ("push", |t| (vec![list_item_type!(t)], VOID_TYPE)),
    ("pop", |t| (vec![], list_item_type!(t))),
    ("len", |_| (vec![], Type::Int)),
    ("is_empty", |_| (vec![], Type::Bool)),
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
