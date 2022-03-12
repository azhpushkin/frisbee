use crate::ast::*;

use super::parser_impl::*;
use crate::tokens::scan_tokens;

type ParsingFunction<T> = fn(&mut Parser) -> ParseResult<T>;

fn parse_helper<T: std::fmt::Debug>(parsefn: ParsingFunction<T>, s: &str) -> T {
    let tokens = scan_tokens(String::from(s));
    let mut parser = Parser::create(tokens);
    let parsed_ast = parsefn(&mut parser);

    assert!(
        parsed_ast.is_ok(),
        "Parse error: {:?}",
        parsed_ast.unwrap_err()
    );
    parsed_ast.unwrap()
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
