use std::collections::HashMap;

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
            types: HashMap::new(),
            functions: HashMap::new(),
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
            types: HashMap::new(),
            functions: HashMap::from([(
                String::from("get_person"),
                FunctionDecl {
                    rettype: Type::TypeBool,
                    name: String::from("get_person"),
                    args: HashMap::from([
                        (
                            "age".into(),
                            TypedNamedObject { typename: Type::TypeInt, name: "age".into() }
                        ),
                        (
                            "name".into(),
                            TypedNamedObject { typename: Type::TypeString, name: "name".into() }
                        ),
                    ]),
                    statements: vec![
                        Statement::SExpr(Expr::ExprBinOp {
                            left: Box::new(Expr::ExprInt(1)),
                            right: Box::new(Expr::ExprMethodCall {
                                object: Box::new(Expr::ExprIdentifier(String::from("asd"))),
                                method: String::from("call"),
                                args: vec![]
                            }),
                            op: BinaryOp::Divide
                        }),
                        Statement::SExpr(Expr::ExprThis)
                    ],
                }
            )])
        }
    );
}

#[test]
fn parse_type_from_import() {
    assert_eq!(
        parse_and_unwrap(
            Parser::parse_top_level,
            "from module.lol import Type; fun Type test() {} "
        ),
        FileAst {
            imports: vec![ImportDecl {
                module_path: ModulePath(vec!["module".into(), "lol".into()]),
                typenames: vec![String::from("Type")],
                functions: vec![]
            },],
            types: HashMap::new(),
            functions: HashMap::from([(
                String::from("test"),
                FunctionDecl {
                    rettype: Type::TypeIdent("Type".into(), ModulePathAlias("module.lol".into())),
                    name: String::from("test"),
                    args: HashMap::new(),
                    statements: vec![],
                }
            )])
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
        ObjectDecl {
            is_active: true,
            name: String::from("Actor"),
            fields: HashMap::from([
                (
                    "lol".into(),
                    TypedNamedObject {
                        typename: Type::TypeIdent(String::from("Actor"), get_test_module_path()),
                        name: "lol".into()
                    }
                ),
                (
                    "name".into(),
                    TypedNamedObject { typename: Type::TypeString, name: "name".into() }
                ),
            ]),
            methods: HashMap::new(),
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
        ObjectDecl {
            is_active: false,
            name: String::from("Data"),
            fields: HashMap::new(),
            methods: HashMap::from([(
                "get_person".into(),
                FunctionDecl {
                    rettype: Type::TypeBool,
                    name: String::from("get_person"),
                    args: HashMap::from([
                        (
                            "age".into(),
                            TypedNamedObject { typename: Type::TypeInt, name: "age".into() }
                        ),
                        (
                            "name".into(),
                            TypedNamedObject { typename: Type::TypeString, name: "name".into() }
                        ),
                    ]),
                    statements: vec![
                        Statement::SExpr(Expr::ExprBinOp {
                            left: Box::new(Expr::ExprInt(1)),
                            right: Box::new(Expr::ExprMethodCall {
                                object: Box::new(Expr::ExprIdentifier(String::from("asd"))),
                                method: String::from("call"),
                                args: vec![]
                            }),
                            op: BinaryOp::Divide
                        }),
                        Statement::SExpr(Expr::ExprThis)
                    ],
                }
            )])
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
        ObjectDecl {
            is_active: false,
            name: String::from("Data"),
            fields: HashMap::new(),
            methods: HashMap::from([(
                "Data".into(),
                FunctionDecl {
                    rettype: Type::TypeIdent(String::from("Data"), get_test_module_path()),
                    name: String::from("Data"),
                    args: HashMap::new(),
                    statements: vec![],
                }
            )])
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
        ObjectDecl {
            is_active: true,
            name: String::from("Actor"),
            fields: HashMap::new(),
            methods: HashMap::from([(
                "Actor".into(),
                FunctionDecl {
                    rettype: Type::TypeIdent(String::from("Actor"), get_test_module_path()),
                    name: String::from("Actor"),
                    args: HashMap::new(),
                    statements: vec![],
                }
            )])
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

#[test]
fn dublicated_methods_are_not_allowed() {
    assert_parsing_fails(
        Parser::parse_top_level,
        "active Actor { fun Nil hello() {} fun Nil hello() {} }",
    );

    // Same object but without duplicated method is fine
    let parsed_ast = parse_helper(
        Parser::parse_top_level,
        "active Actor { fun Nil hello() {} fun Nil hello2() {} }",
    );
    assert!(parsed_ast.is_ok());
}

#[test]
fn dublicated_fields_are_not_allowed() {
    assert_parsing_fails(
        Parser::parse_top_level,
        "active Actor { Bool lol; Int lol; }",
    );

    // Same object but without duplicated method is fine
    let parsed_ast = parse_helper(
        Parser::parse_top_level,
        "active Actor { Bool lol; Int lol_; }",
    );
    assert!(parsed_ast.is_ok());
}

#[test]
fn dublicated_args_in_function_are_not_allowed() {
    assert_parsing_fails(Parser::parse_top_level, "fun Nil hello(Int a, Bool a) {}");

    // Same object but without duplicated method is fine
    let parsed_ast = parse_helper(Parser::parse_top_level, "fun Nil hello(Int a, Bool a_) {}");
    assert!(parsed_ast.is_ok());
}

#[test]
fn dublicated_args_in_method_are_not_allowed() {
    assert_parsing_fails(
        Parser::parse_top_level,
        "active Actor { fun Nil hello(Int a, Bool a) {} }",
    );

    // Same object but without duplicated method is fine
    let parsed_ast = parse_helper(
        Parser::parse_top_level,
        "active Actor { fun Nil hello(Int a, Bool a_) {} }",
    );
    assert!(parsed_ast.is_ok());
}
