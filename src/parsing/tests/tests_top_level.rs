use crate::ast::parsed::*;
use crate::types::Type;

use super::super::parser::*;
use super::tests_helpers::*;

#[test]
fn simple_import() {
    assert_eq!(
        parse_and_unwrap(|p| Parser::parse_import(p), "from module import Actor;"),
        ImportDecl {
            pos: 0,
            module_path: vec!["module".into()],
            typenames: vec![String::from("Actor")],
            functions: vec![]
        }
    );

    assert_eq!(
        parse_and_unwrap(|p| Parser::parse_import(p), "from module.sub import func;"),
        ImportDecl {
            pos: 0,
            module_path: vec!["module".into(), "sub".into()],
            typenames: vec![],
            functions: vec!["func".into()]
        }
    );

    assert_parsing_fails(|p| Parser::parse_import(p), "from module import Type");
    assert_parsing_fails(|p| Parser::parse_import(p), "from module. import Type;");
    assert_parsing_fails(|p| Parser::parse_import(p), "from module.123 import Type");
}

#[test]
fn simple_import_of_functions() {
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_import(p),
            "from module import func, Type, f;"
        ),
        ImportDecl {
            pos: 0,
            module_path: vec!["module".into()],
            typenames: vec![String::from("Type")],
            functions: vec!["func".into(), "f".into()]
        }
    );
}

#[test]
fn multiple_imports() {
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_top_level(p),
            "from some2 import Hello, There; from two import One;"
        ),
        FileAst {
            imports: vec![
                ImportDecl {
                    pos: 0,
                    module_path: vec![String::from("some2")],
                    typenames: vec![String::from("Hello"), String::from("There")],
                    functions: vec![]
                },
                ImportDecl {
                    pos: 32,
                    module_path: vec![String::from("two")],
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
    let var_decl_statement = Statement::VarDeclWithAssign(
        Type::Int,
        "var".into(),
        ExprWithPos {
            expr: Expr::FunctionCall {
                function: "asd".into(),
                args: vec![ExprWithPos {
                    expr: Expr::String("lol".into()),
                    pos_first: 58,
                    pos_last: 62,
                }],
            },
            pos_first: 54,
            pos_last: 63,
        },
    );
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_top_level(p),
            r#"fun void get_person(Int age, String name) { Int var = asd("lol"); } "#
        ),
        FileAst {
            imports: vec![],
            types: vec![],
            functions: vec![FunctionDecl {
                pos: 0,
                rettype: None,
                name: String::from("get_person"),
                args: vec![
                    TypedItem { typename: Type::Int, name: "age".into() },
                    TypedItem { typename: Type::String, name: "name".into() }
                ],
                statements: vec![StatementWithPos { statement: var_decl_statement, pos: 44 }],
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
            pos: 0,
            is_active: true,
            name: String::from("Actor"),
            fields: vec![
                TypedItem { typename: Type::String, name: "name".into() },
                TypedItem { typename: Type::Custom(String::from("Actor")), name: "lol".into() },
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
            "class Data { fun Bool get_person(Int age, String name) { this; } }"
        ),
        ClassDecl {
            pos: 0,
            is_active: false,
            name: String::from("Data"),
            fields: vec![],
            methods: vec![FunctionDecl {
                pos: 13,
                rettype: Some(Type::Bool),
                name: String::from("get_person"),
                args: vec![
                    TypedItem { typename: Type::Int, name: "age".into() },
                    TypedItem { typename: Type::String, name: "name".into() },
                ],
                statements: vec![StatementWithPos {
                    statement: Statement::Expr(ExprWithPos {
                        expr: Expr::This,
                        pos_first: 57,
                        pos_last: 60
                    }),
                    pos: 57
                }],
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
            pos: 0,
            is_active: false,
            name: String::from("Data"),
            fields: vec![],
            methods: vec![FunctionDecl {
                pos: 13,
                rettype: Some(Type::Custom(String::from("Data"))),
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
            pos: 0,
            is_active: true,
            name: String::from("Actor"),
            fields: vec![],
            methods: vec![FunctionDecl {
                pos: 15,
                rettype: Some(Type::Custom(String::from("Actor"))),
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
