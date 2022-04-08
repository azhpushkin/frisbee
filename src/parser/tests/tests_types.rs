use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

fn assert_type_parses(s: &str, t: Type) {
    assert_eq!(parse_and_unwrap(Parser::parse_type, s), t);
}

#[test]
fn simple_types() {
    assert_type_parses("String", Type::String);
    assert_type_parses("Int", Type::Int);
    assert_type_parses("Float", Type::Float);
    assert_type_parses("Nil", Type::Nil);
    assert_type_parses("Bool", Type::Bool);

    assert_type_parses("StriNG", Type::Ident(String::from("StriNG")));
    assert_type_parses("SomeClass", Type::Ident(String::from("SomeClass")));
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
    assert_type_parses("[String]", Type::List(Box::new(Type::String)));
    assert_type_parses(
        "[[Actor]]",
        Type::List(Box::new(Type::List(Box::new(Type::Ident(
            String::from("Actor"),
        ))))),
    );

    assert_parsing_fails(Parser::parse_type, "[ ]");
    assert_parsing_fails(Parser::parse_type, "[Int, String]");
    assert_parsing_fails(Parser::parse_type, "[Int, ]");
}

#[test]
fn maybe_types() {
    assert_type_parses("String?", Type::Maybe(Box::new(Type::String)));
    assert_type_parses(
        "[Actor?]?",
        Type::Maybe(Box::new(Type::List(Box::new(Type::Maybe(
            Box::new(Type::Ident("Actor".into())),
        ))))),
    );
}

#[test]
fn tuple_types() {
    assert_type_parses(
        "(String, Int)",
        Type::Tuple(vec![Type::String, Type::Int]),
    );
    assert_type_parses(
        "(Actor, (Nil, Bool, Class, Passive), Int)",
        Type::Tuple(vec![
            Type::Ident(String::from("Actor")),
            Type::Tuple(vec![
                Type::Nil,
                Type::Bool,
                Type::Ident(String::from("Class")),
                Type::Ident(String::from("Passive")),
            ]),
            Type::Int,
        ]),
    );

    // Allow trailing commas here
    assert_type_parses(
        "(String, Int, )",
        Type::Tuple(vec![Type::String, Type::Int]),
    );

    // Single element tuple is shrinked to just that element
    assert_type_parses("(String)", Type::String);
    assert_type_parses("(Actor, )", Type::Ident(String::from("Actor")));

    // Empty tuple is not allowed
    assert_parsing_fails(Parser::parse_type, "()");
}

#[test]
fn complex_types_maybe_and_list_order() {
    assert_type_parses(
        "[Int]?",
        Type::Maybe(Box::new(Type::List(Box::new(Type::Int)))),
    );

    assert_type_parses(
        "[Int?]",
        Type::List(Box::new(Type::Maybe(Box::new(Type::Int)))),
    );
}

#[test]
fn complex_types() {
    assert_type_parses(
        "(Actor?, [Bool])",
        Type::Tuple(vec![
            Type::Maybe(Box::new(Type::Ident(String::from("Actor")))),
            Type::List(Box::new(Type::Bool)),
        ]),
    );
}

#[test]
fn very_complex_types() {
    assert_type_parses(
        "[( [(Bool, Int?)]?, String )]",
        Type::List(Box::new(Type::Tuple(vec![
            Type::Maybe(Box::new(Type::List(Box::new(Type::Tuple(vec![
                Type::Bool,
                Type::Maybe(Box::new(Type::Int)),
            ]))))),
            Type::String,
        ]))),
    );
}
