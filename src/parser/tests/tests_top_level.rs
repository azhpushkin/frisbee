use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

#[test]
fn simple_import() {
    assert_eq!(
        parse_and_unwrap(Parser::parse_import, "from module import Actor;"),
        ImportDecl {
            module_path: ModulePath(vec!["module".into()]),
            typenames: vec![String::from("Actor")],
            functions: vec![]
        }
    );

    assert_eq!(
        parse_and_unwrap(Parser::parse_import, "from module.sub import func;"),
        ImportDecl {
            module_path: ModulePath(vec!["module".into(), "sub".into()]),
            typenames: vec![],
            functions: vec!["func".into()]
        }
    );

    assert_parsing_fails(Parser::parse_import, "from module import Type");
    assert_parsing_fails(Parser::parse_import, "from module. import Type;");
    assert_parsing_fails(Parser::parse_import, "from module.123 import Type");
}

#[test]
fn simple_import_of_functions() {
    assert_eq!(
        parse_and_unwrap(Parser::parse_import, "from module import func, Type, f;"),
        ImportDecl {
            module_path: ModulePath(vec!["module".into()]),
            typenames: vec![String::from("Type")],
            functions: vec!["func".into(), "f".into()]
        }
    );
}

#[test]
fn multiple_imports() {
    assert_eq!(
        parse_and_unwrap(
            Parser::parse_top_level,
            "from some2 import Hello, There; from two import One;"
        ),
        FileAst {
            imports: vec![
                ImportDecl {
                    module_path: ModulePath(vec![String::from("some2")]),
                    typenames: vec![String::from("Hello"), String::from("There")],
                    functions: vec![]
                },
                ImportDecl {
                    module_path: ModulePath(vec![String::from("two")]),
                    typenames: vec![String::from("One")],
                    functions: vec![]
                }
            ],
            types: vec![],
            functions: vec![],
        }
    );
}

#[test]
fn parse_function_definition() {
    assert_eq!(
        parse_and_unwrap(
            Parser::parse_top_level,
            "fun Bool get_person(Int age, String name) { 1 / asd.call(); this; } "
        ),
        FileAst {
            imports: vec![],
            types: vec![],
            functions: vec![FunctionDecl {
                rettype: Type::Bool,
                name: String::from("get_person"),
                args: vec![
                    TypedNamedObject { typename: Type::Int, name: "age".into() },
                    TypedNamedObject { typename: Type::String, name: "name".into() }
                ],
                statements: vec![
                    Statement::Expr(Expr::BinOp {
                        left: Box::new(Expr::Int(1)),
                        right: Box::new(Expr::MethodCall {
                            object: Box::new(Expr::Identifier(String::from("asd"))),
                            method: String::from("call"),
                            args: vec![]
                        }),
                        op: BinaryOp::Divide
                    }),
                    Statement::Expr(Expr::This)
                ],
            }]
        }
    );
}

#[test]
fn active_object_and_fields() {
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_object(p, true),
            "active Actor { String name; Actor lol; }"
        ),
        ClassDecl {
            is_active: true,
            name: String::from("Actor"),
            fields: vec![
                TypedNamedObject { typename: Type::String, name: "name".into() },
                TypedNamedObject {
                    typename: Type::Ident(String::from("Actor")),
                    name: "lol".into()
                },
            ],
            methods: vec![],
        }
    );
}

#[test]
fn class_object_and_methods() {
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_object(p, false),
            "class Data { fun Bool get_person(Int age, String name) { 1 / asd.call(); this; } }"
        ),
        ClassDecl {
            is_active: false,
            name: String::from("Data"),
            fields: vec![],
            methods: vec![FunctionDecl {
                rettype: Type::Bool,
                name: String::from("get_person"),
                args: vec![
                    TypedNamedObject { typename: Type::Int, name: "age".into() },
                    TypedNamedObject { typename: Type::String, name: "name".into() },
                ],
                statements: vec![
                    Statement::Expr(Expr::BinOp {
                        left: Box::new(Expr::Int(1)),
                        right: Box::new(Expr::MethodCall {
                            object: Box::new(Expr::Identifier(String::from("asd"))),
                            method: String::from("call"),
                            args: vec![]
                        }),
                        op: BinaryOp::Divide
                    }),
                    Statement::Expr(Expr::This)
                ],
            }]
        }
    );
}

#[test]
fn class_object_constructor_method() {
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_object(p, false),
            "class Data { fun Data() {} }"
        ),
        ClassDecl {
            is_active: false,
            name: String::from("Data"),
            fields: vec![],
            methods: vec![FunctionDecl {
                rettype: Type::Ident(String::from("Data")),
                name: String::from("Data"),
                args: vec![],
                statements: vec![],
            }]
        }
    );

    // spawn is not allowed for classes
    assert_parsing_fails(
        |p| Parser::parse_object(p, false),
        "struct Data { fun Data Data() {} }",
    );
    assert_parsing_fails(
        |p| Parser::parse_object(p, false),
        "struct Data { fun Data DataConstructor() {} }",
    );
}

#[test]
fn active_object_constructor_method() {
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_object(p, true),
            "active Actor { fun Actor() {} }"
        ),
        ClassDecl {
            is_active: true,
            name: String::from("Actor"),
            fields: vec![],
            methods: vec![FunctionDecl {
                rettype: Type::Ident(String::from("Actor")),
                name: String::from("Actor"),
                args: vec![],
                statements: vec![],
            }]
        }
    );

    assert_parsing_fails(
        |p| Parser::parse_object(p, true),
        "active Actor { fun Actor Actor() {} }",
    );
    assert_parsing_fails(
        |p| Parser::parse_object(p, true),
        "active Actor { fun ActorConstructor() {} }",
    );
}