use crate::ast::*;

use super::parser_impl::*;
use super::tests_helpers::parse_and_unwrap;

#[test]
fn simple_import() {
    assert_eq!(
        parse_and_unwrap(Parser::parse_import, "from module import Actor;"),
        ImportDecl { module: String::from("module"), typenames: vec![String::from("Actor")] }
    );
}

#[test]
fn multiple_imports() {
    assert_eq!(
        parse_and_unwrap(
            Parser::parse_top_level,
            "from some2 import Hello, There; from two import One;"
        ),
        Program {
            imports: vec![
                ImportDecl {
                    module: String::from("some2"),
                    typenames: vec![String::from("Hello"), String::from("There")]
                },
                ImportDecl { module: String::from("two"), typenames: vec![String::from("One")] }
            ],
            passive: vec![],
            active: vec![]
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
fn passive_object_and_methods() {
    assert_eq!(
        parse_and_unwrap(
            |p| Parser::parse_object(p, false),
            "passive Data { fun Bool get_person(Int age, String name) { 1 / asd.call(); this; } }"
        ),
        ObjectDecl {
            is_active: false,
            name: String::from("Data"),
            fields: vec![],
            methods: vec![MethodDecl {
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
