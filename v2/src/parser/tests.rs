use crate::ast::*;

use super::parser_impl::*;
use crate::tokens::scan_tokens;

type ParsingFunction<T> = fn(&mut Parser) -> ParseResult<T>;

fn parse_helper<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) -> T {
    let tokens = scan_tokens(&String::from(s));
    let mut parser = Parser::create(tokens);
    let parsed_ast = parsefn(&mut parser);

    assert!(
        parsed_ast.is_ok(),
        "Parse error: {:?}",
        parsed_ast.unwrap_err()
    );
    parsed_ast.unwrap()
}

fn assert_type_parsing_fails<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) {
    let tokens = scan_tokens(&String::from(s));
    let mut parser = Parser::create(tokens);
    let parsed_ast = parsefn(&mut parser);

    assert!(
        parsed_ast.is_err(),
        "Parsed to: {:?}",
        parsed_ast.unwrap()
    );
}

fn assert_type_parses(s: &str, t: Type) {
    assert_eq!(parse_helper(Parser::parse_type, s), t);
}

fn assert_expr_parses(s: &str, t: Expr) {
    assert_eq!(parse_helper(Parser::parse_expr, s), t);
}

#[test]
fn simple_import() {
    assert_eq!(
        parse_helper(Parser::parse_import, "from module import Actor;"),
        ImportDecl { module: String::from("module"), typenames: vec![String::from("Actor")] }
    );
}

#[test]
fn multiple_imports() {
    assert_eq!(
        parse_helper(
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
        parse_helper(
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
        parse_helper(
            |p| Parser::parse_object(p, false),
            "passive Data { fun Bool get_person(Int age, String name) {} }"
        ),
        ObjectDecl {
            is_active: false,
            name: String::from("Data"),
            fields: vec![],
            methods: vec![MethodDecl{
                rettype: Type::TypeBool,
                name: String::from("get_person"),
                args: vec![
                    TypedNamedObject { typename: Type::TypeInt, name: String::from("age") },
                    TypedNamedObject {
                        typename: Type::TypeString,
                        name: String::from("name")
                    },
                ],
                statements: vec![],

            }]
        }
    );
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
    assert_type_parsing_fails(Parser::parse_type, "string");
    assert_type_parsing_fails(Parser::parse_type, "int");
    assert_type_parsing_fails(Parser::parse_type, "asd");

    assert_type_parsing_fails(Parser::parse_type, "?");
    assert_type_parsing_fails(Parser::parse_type, "[String");
    assert_type_parsing_fails(Parser::parse_type, "[[String]");
    assert_type_parsing_fails(Parser::parse_type, "(String(");
    assert_type_parsing_fails(Parser::parse_type, ")String");
    assert_type_parsing_fails(Parser::parse_type, ")String");
    
    
}

#[test]
fn list_types() {
    assert_type_parses("[String]", Type::TypeList(Box::new(Type::TypeString)));
    assert_type_parses(
        "[[Actor]]",
        Type::TypeList(Box::new(
            Type::TypeList(Box::new(
                Type::TypeIdent(String::from("Actor"))
            ))
        ))
    );

    assert_type_parsing_fails(Parser::parse_type, "[ ]");
    assert_type_parsing_fails(Parser::parse_type, "[Int, String]");
    assert_type_parsing_fails(Parser::parse_type, "[Int, ]");
}

#[test]
fn maybe_types() {
    assert_type_parses("String?", Type::TypeMaybe(Box::new(Type::TypeString)));
    assert_type_parses(
        "Actor??",
        Type::TypeMaybe(Box::new(
            Type::TypeMaybe(Box::new(
                Type::TypeIdent(String::from("Actor"))
            ))
        ))
    );
}

#[test]
fn tuple_types() {
    assert_type_parses(
        "(String, Int)",
        Type::TypeTuple(vec![Type::TypeString, Type::TypeInt])
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
            Type::TypeInt
        ])
    );

    // Allow trailing commas here
    assert_type_parses(
        "(String, Int, )",
        Type::TypeTuple(vec![Type::TypeString, Type::TypeInt])
    );

    // Single element tuple is shrinked to just that element
    assert_type_parses("(String)", Type::TypeString);
    assert_type_parses("(Actor, )", Type::TypeIdent(String::from("Actor")));

    // Empty tuple is not allowed
    assert_type_parsing_fails(Parser::parse_type, "()");
}


#[test]
fn complex_types_maybe_and_list_order() {
    assert_type_parses(
        "[Int]?",
        Type::TypeMaybe(Box::new(
            Type::TypeList(Box::new(
                Type::TypeInt
            )),
        )),
    );

    assert_type_parses(
        "[Int?]",
        Type::TypeList(Box::new(
            Type::TypeMaybe(Box::new(
                Type::TypeInt
            )),
        )),
    );
}

#[test]
fn complex_types() {
    assert_type_parses(
        "(Actor?, [Bool])",
        Type::TypeTuple(vec![
            Type::TypeMaybe(Box::new(
                Type::TypeIdent(String::from("Actor"))
            )),
            Type::TypeList(Box::new(
                Type::TypeBool
            )),
        ])
    );
}

#[test]
fn very_complex_types() {
    assert_type_parses(
        "[( [(Bool, Int?)]?, String )]",
        Type::TypeList(Box::new(
            Type::TypeTuple(vec![
                Type::TypeMaybe(Box::new(
                    Type::TypeList(Box::new(
                        Type::TypeTuple(vec![
                            Type::TypeBool,
                            Type::TypeMaybe(Box::new(
                                Type::TypeInt
                            ))
                        ])
                    )),
                )),
                Type::TypeString
            ])
        )),
    );
}

#[test]
fn operator_simple_equality() {
    assert_expr_parses(
        "nil == asd",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprNil),
            right: Box::new(Expr::ExprIdentifier(String::from("asd"))),
            op: BinaryOp::IsEqual
        }
    );
}

#[test]
fn operator_expression() {
    assert_expr_parses(
        "1 + asd",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprInt(1)),
            right: Box::new(Expr::ExprIdentifier(String::from("asd"))),
            op: BinaryOp::Plus 
        }
    );

    assert_expr_parses(
        r#"- "hello""#, 
        Expr::ExprUnaryOp {
            operand: Box::new(Expr::ExprString(String::from("hello"))),
            op: UnaryOp::Negate
        }
    );

    assert_expr_parses(
        "true / 23.2",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprBool(true)),
            right: Box::new(Expr::ExprFloat(23.2)),
            op: BinaryOp::Divide
        }
    )
}

#[test]
fn expr_minus_minus() {
    assert_expr_parses(
        "-1.0- - 2",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprUnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::ExprFloat(1.0)),
            }),
            right: Box::new(Expr::ExprUnaryOp {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::ExprInt(2)),
            }),
            op: BinaryOp::Minus 
        }
    )
}

#[test]
fn expr_operator_order() {
    assert_expr_parses(
        "1 + 2 * qw2",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprInt(1)),
            right: Box::new(Expr::ExprBinOp{
                left: Box::new(Expr::ExprInt(2)),
                right: Box::new(Expr::ExprIdentifier(String::from("qw2"))),
                op: BinaryOp::Multiply
            }),
            op: BinaryOp::Plus
        }
        
    )
}

#[test]
fn expr_operator_order_with_grouping() {
    assert_expr_parses(
        "1 + (2 * qw2)",
        Expr::ExprBinOp {
            left: Box::new(Expr::ExprInt(1)),
            right: Box::new(Expr::ExprBinOp{
                left: Box::new(Expr::ExprInt(2)),
                right: Box::new(Expr::ExprIdentifier(String::from("qw2"))),
                op: BinaryOp::Plus
            }),
            op: BinaryOp::Plus
        }
        
    )
}

#[test]
fn expr_function_call() {
    assert!(false);  // todo
}




// TODO: test associativyty
// TODO: test function call
// TODO: test array access
// TODO: test array 



// statements
// TODO: test array assignment
// TODO 
