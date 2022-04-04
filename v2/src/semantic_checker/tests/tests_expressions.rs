use std::collections::HashMap;

use crate::ast::{Expr, ModulePath, Type};
use crate::parser::parser_impl::Parser;
use crate::scanner::scan_tokens;
use crate::semantic_checker::expressions::ExprTypeChecker;
use crate::test_utils::{new_alias, setup_and_load_program};

use super::super::modules::*;
use super::super::type_env::TypeEnv;

const example_program: &str = r#"
===== file: main.frisbee
class Person {
    String name;
    Int? age;
    
    fun [String] get_nicknames() {}
    fun Bool is_adult() {}
}

active Worker {
    String id;
}

// varibles, defined in setup_env:
// Worker worker
// Person alice
// Person? bob
// [String] cli_args
"#;

fn setup_maps(use_scope: bool) -> (SymbolOriginsPerFile, GlobalSignatures) {
    let wp = setup_and_load_program(example_program);
    let file = wp.files.get(&ModulePath(vec!["main".into()]).alias()).unwrap();
    let origins = SymbolOriginsPerFile {
        typenames: get_typenames_mapping(file).unwrap(),
        functions: get_functions_mapping(file).unwrap(),
    };
    let signatures = GlobalSignatures {
        typenames: get_typenames_signatures(file, &origins.typenames).unwrap(),
        functions: get_functions_signatures(file, &origins.typenames).unwrap(),
    };
    (origins, signatures)
}

fn setup_env<'a>(
    use_scope: bool,
    s: &'a SymbolOriginsPerFile,
    g: &'a GlobalSignatures,
) -> TypeEnv<'a> {
    let person_type = Type::TypeIdentQualified(new_alias("main"), "Person".into());

    TypeEnv {
        variables_types: HashMap::from([
            ("alice".into(), person_type.clone()),
            ("bob".into(), Type::TypeMaybe(Box::new(person_type))),
            (
                "cli_args".into(),
                Type::TypeList(Box::new(Type::TypeString)),
            ),
            (
                "worker".into(),
                Type::TypeIdentQualified(new_alias("main"), "Worker".into()),
            ),
        ]),
        symbol_origins: s,
        signatures: g,
        scope: if use_scope {
            Some((new_alias("main"), "Person".into()))
        } else {
            None
        },
    }
}

fn parse_expr(expr_string: &str) -> Expr {
    let tokens = scan_tokens(&expr_string.into());
    let mut parser = Parser::create(tokens.expect("Scanning failed!"));
    parser.parse_expr().expect("Parsing failed!")
}

fn assert_expr_ok(expr_str: &str, use_scope: bool, expected_type: Type) {
    let expr = parse_expr(expr_str);
    let (s, g) = setup_maps(use_scope);
    let env = setup_env(use_scope, &s, &g);
    let checker = ExprTypeChecker::new(&env);
    let res = checker.calculate(&expr);
    assert!(res.is_ok(), "Typecheck failed: {}", res.unwrap_err());
    assert_eq!(res.unwrap(), expected_type);
}

fn assert_expr_fails(expr_str: &str, use_scope: bool) {
    let expr = parse_expr(expr_str);
    let (s, g) = setup_maps(use_scope);
    let env = setup_env(use_scope, &s, &g);
    let checker = ExprTypeChecker::new(&env);
    let res = checker.calculate(&expr);
    assert!(
        res.is_err(),
        "Typecheck HAD TO FAIL, BUT resulted in : {:?}",
        res.unwrap()
    );
}

#[test]
fn test_simple_operator() {
    assert_expr_ok("1 + 1", false, Type::TypeInt);
    assert_expr_ok("2.0 + 0.0", false, Type::TypeFloat);

    assert_expr_fails("2.0 + \"hello\" ", false);
}

// NO SCOPE
// 1+1 -> Int
// 1.0 + 1.0 -> Float
// [1, 2] + [1, 2] -> [Int]
// [2, 2] + ["asd"]  ERROR
// "asd" + "asd" String
