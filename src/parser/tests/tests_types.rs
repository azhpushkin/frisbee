use super::super::parser_impl::*;
use super::tests_helpers::*;

type T = crate::types::ParsedType;

fn assert_type_parses(s: &str, t: T) {
    assert_eq!(parse_and_unwrap(Parser::parse_type, s), t);
}

#[test]
fn simple_types() {
    assert_type_parses("String", T::String);
    assert_type_parses("Int", T::Int);
    assert_type_parses("Float", T::Float);

    assert_type_parses("Bool", T::Bool);

    assert_type_parses("StriNG", T::Custom(String::from("StriNG")));
    assert_type_parses("SomeClass", T::Custom(String::from("SomeClass")));
}

#[test]
fn types_parsing_errors() {
    assert_parsing_fails(Parser::parse_type, "string");
    assert_parsing_fails(Parser::parse_type, "int");
    assert_parsing_fails(Parser::parse_type, "asd");

    assert_parsing_fails(Parser::parse_type, "?");
    assert_parsing_fails(Parser::parse_type, "[String");
    assert_parsing_fails(Parser::parse_type, "[[String]");
    assert_parsing_fails(Parser::parse_type, "(String(");
    assert_parsing_fails(Parser::parse_type, ")String");
    assert_parsing_fails(Parser::parse_type, ")String");
}

#[test]
fn list_types() {
    assert_type_parses("[String]", T::List(Box::new(T::String)));
    assert_type_parses(
        "[[Actor]]",
        T::List(Box::new(T::List(Box::new(T::Custom(String::from(
            "Actor",
        )))))),
    );

    assert_parsing_fails(Parser::parse_type, "[ ]");
    assert_parsing_fails(Parser::parse_type, "[Int, String]");
    assert_parsing_fails(Parser::parse_type, "[Int, ]");
}

#[test]
fn maybe_types() {
    assert_type_parses("String?", T::Maybe(Box::new(T::String)));
    assert_type_parses(
        "[Actor?]?",
        T::Maybe(Box::new(T::List(Box::new(T::Maybe(Box::new(T::Custom(
            "Actor".into(),
        ))))))),
    );
}

#[test]
fn tuple_types() {
    assert_type_parses("(String, Int)", T::Tuple(vec![T::String, T::Int]));
    assert_type_parses(
        "(Actor, (Bool, Class, Passive), Int)",
        T::Tuple(vec![
            T::Custom(String::from("Actor")),
            T::Tuple(vec![
                T::Bool,
                T::Custom(String::from("Class")),
                T::Custom(String::from("Passive")),
            ]),
            T::Int,
        ]),
    );

    // Allow trailing commas here
    assert_type_parses("(String, Int, )", T::Tuple(vec![T::String, T::Int]));

    // Single element tuple is shrinked to just that element
    assert_type_parses("(String)", T::String);
    assert_type_parses("(Actor, )", T::Custom(String::from("Actor")));

    // Empty tuple is not allowed
    assert_parsing_fails(Parser::parse_type, "()");
}

#[test]
fn complex_types_maybe_and_list_order() {
    assert_type_parses("[Int]?", T::Maybe(Box::new(T::List(Box::new(T::Int)))));

    assert_type_parses("[Int?]", T::List(Box::new(T::Maybe(Box::new(T::Int)))));
}

#[test]
fn complex_types() {
    assert_type_parses(
        "(Actor?, [Bool])",
        T::Tuple(vec![
            T::Maybe(Box::new(T::Custom(String::from("Actor")))),
            T::List(Box::new(T::Bool)),
        ]),
    );
}

#[test]
fn very_complex_types() {
    assert_type_parses(
        "[( [(Bool, Int?)]?, String )]",
        T::List(Box::new(T::Tuple(vec![
            T::Maybe(Box::new(T::List(Box::new(T::Tuple(vec![
                T::Bool,
                T::Maybe(Box::new(T::Int)),
            ]))))),
            T::String,
        ]))),
    );
}
