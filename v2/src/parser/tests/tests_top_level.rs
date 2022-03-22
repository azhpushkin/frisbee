use crate::ast::*;

use super::super::parser_impl::*;
use super::tests_helpers::*;

#[test]
fn simple_import() {
    assert_eq!(
        parse_and_unwrap(Parser::parse_import, "from module import Actor;"),
        ImportDecl { module_path: vec!["module".into()], typenames: vec![String::from("Actor")] , functions: vec![]}
    );
}

#[test]
fn simple_import_of_functions() {
    assert_eq!(
        parse_and_unwrap(Parser::parse_import, "from module import func, Type, f;"),
        ImportDecl { module_path: vec!["module".into()], typenames: vec![String::from("Type")] , functions: vec!["func".into(), "f".into()]}
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
                    module_path: vec![String::from("some2")],
                    typenames: vec![String::from("Hello"), String::from("There")],
                    functions: vec![]
                },
                ImportDecl { module_path: vec![String::from("two")], typenames: vec![String::from("One")] ,functions: vec![]}
            ],
            classes: vec![],
            functions: vec![],
            active: vec![]
        }
    );
}


#[test]
fn function_() {
    assert_eq!(
        parse_and_unwrap(
            Parser::parse_top_level,
            "fun Bool get_person(Int age, String name) { 1 / asd.call(); this; } "
        ),
        FileAst {
            imports: vec![],
            classes: vec![],
            active: vec![],
            functions: vec![FunctionDecl {
                rettype: Type::TypeBool,
                name: String::from("get_person"),
                args: vec![
                    TypedNamedObject { typename: Type::TypeInt, name: String::from("age") },
                    TypedNamedObject { typename: Type::TypeString, name: String::from("name") },
                ],
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
        ObjectDecl {
            is_active: true,
            name: String::from("Actor"),
            fields: vec![
                TypedNamedObject { typename: Type::TypeString, name: String::from("name") },
                TypedNamedObject {
                    typename: Type::TypeIdent(String::from("Actor")),
                    name: String::from("lol")
                },
            ],
            methods: vec![]
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
            fields: vec![],
            methods: vec![FunctionDecl {
                rettype: Type::TypeBool,
                name: String::from("get_person"),
                args: vec![
                    TypedNamedObject { typename: Type::TypeInt, name: String::from("age") },
                    TypedNamedObject { typename: Type::TypeString, name: String::from("name") },
                ],
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
        ObjectDecl {
            is_active: false,
            name: String::from("Data"),
            fields: vec![],
            methods: vec![FunctionDecl {
                rettype: Type::TypeIdent(String::from("Data")),
                name: String::from("Data"),
                args: vec![],
                statements: vec![],
            }]
        }
    );

    // spawn is not allowed for classes
    assert_parsing_fails(
        |p| Parser::parse_object(p, false),
        "struct Data { fun Data Data() {} }"
    );
    assert_parsing_fails(
        |p| Parser::parse_object(p, false),
        "struct Data { fun Data DataConstructor() {} }"
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
            fields: vec![],
            methods: vec![FunctionDecl {
                rettype: Type::TypeIdent(String::from("Actor")),
                name: String::from("Actor"),
                args: vec![],
                statements: vec![],
            }]
        }
    );

    assert_parsing_fails(
        |p| Parser::parse_object(p, true),
        "active Actor { fun Actor Actor() {} }"
    );
    assert_parsing_fails(
        |p| Parser::parse_object(p, true),
        "active Actor { fun ActorConstructor() {} }"
    );
}



#[test]
fn dublicated_methods_are_not_allowed() {
    assert_parsing_fails(
        |p| Parser::parse_object(p, true),
        "active Actor { fun Nil hello() {} fun Nil hello() {} }"
    );

    // Same object but without duplicated method is fine
    let parsed_ast = parse_helper(
        |p| Parser::parse_object(p, true),
        "active Actor { fun Nil hello() {} fun Nil hello2() {} }"
    );
    assert!(parsed_ast.is_ok());
}
