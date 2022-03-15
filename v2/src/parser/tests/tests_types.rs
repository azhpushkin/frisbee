use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

fn assert_type_parses(s: &str, t: Type) {
    assert_eq!(parse_and_unwrap(Parser::parse_type, s), t);
}

#[test]
fn simple_types() {
    assert_type_parses("String", Type::TypeString);
    assert_type_parses("Int", Type::TypeInt);
    assert_type_parses("Float", Type::TypeFloat);
    assert_type_parses("Nil", Type::TypeNil);
    assert_type_parses("Bool", Type::TypeBool);

    assert_type_parses("StriNG", Type::TypeIdent(String::from("StriNG")));
    assert_type_parses("SomeClass", Type::TypeIdent(String::from("SomeClass")));
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
    assert_type_parses("[String]", Type::TypeList(Box::new(Type::TypeString)));
    assert_type_parses(
        "[[Actor]]",
        Type::TypeList(Box::new(Type::TypeList(Box::new(Type::TypeIdent(
            String::from("Actor"),
        ))))),
    );

    assert_parsing_fails(Parser::parse_type, "[ ]");
    assert_parsing_fails(Parser::parse_type, "[Int, String]");
    assert_parsing_fails(Parser::parse_type, "[Int, ]");
}

#[test]
fn maybe_types() {
    assert_type_parses("String?", Type::TypeMaybe(Box::new(Type::TypeString)));
    assert_type_parses(
        "Actor??",
        Type::TypeMaybe(Box::new(Type::TypeMaybe(Box::new(Type::TypeIdent(
            String::from("Actor"),
        ))))),
    );
}

#[test]
fn tuple_types() {
    assert_type_parses(
        "(String, Int)",
        Type::TypeTuple(vec![Type::TypeString, Type::TypeInt]),
    );
    assert_type_parses(
        "(Actor, (Nil, Bool, Passive), Int)",
        Type::TypeTuple(vec![
            Type::TypeIdent(String::from("Actor")),
            Type::TypeTuple(vec![
                Type::TypeNil,
                Type::TypeBool,
                Type::TypeIdent(String::from("Passive")),
            ]),
            Type::TypeInt,
        ]),
    );

    // Allow trailing commas here
    assert_type_parses(
        "(String, Int, )",
        Type::TypeTuple(vec![Type::TypeString, Type::TypeInt]),
    );

    // Single element tuple is shrinked to just that element
    assert_type_parses("(String)", Type::TypeString);
    assert_type_parses("(Actor, )", Type::TypeIdent(String::from("Actor")));

    // Empty tuple is not allowed
    assert_parsing_fails(Parser::parse_type, "()");
}

#[test]
fn complex_types_maybe_and_list_order() {
    assert_type_parses(
        "[Int]?",
        Type::TypeMaybe(Box::new(Type::TypeList(Box::new(Type::TypeInt)))),
    );

    assert_type_parses(
        "[Int?]",
        Type::TypeList(Box::new(Type::TypeMaybe(Box::new(Type::TypeInt)))),
    );
}

#[test]
fn complex_types() {
    assert_type_parses(
        "(Actor?, [Bool])",
        Type::TypeTuple(vec![
            Type::TypeMaybe(Box::new(Type::TypeIdent(String::from("Actor")))),
            Type::TypeList(Box::new(Type::TypeBool)),
        ]),
    );
}

#[test]
fn very_complex_types() {
    assert_type_parses(
        "[( [(Bool, Int?)]?, String )]",
        Type::TypeList(Box::new(Type::TypeTuple(vec![
            Type::TypeMaybe(Box::new(Type::TypeList(Box::new(Type::TypeTuple(vec![
                Type::TypeBool,
                Type::TypeMaybe(Box::new(Type::TypeInt)),
            ]))))),
            Type::TypeString,
        ]))),
    );
}
