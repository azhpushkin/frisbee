use crate::ast::{Expr, Type};
use crate::parser::parser_impl::Parser; // TODO: do not depend on parser here!
use crate::scanner::scan_tokens;
use crate::semantic_checker::check_and_annotate_symbols;
use crate::semantic_checker::expressions::ExprTypeChecker;
use crate::semantic_checker::symbols::GlobalSymbolsInfo;
use crate::test_utils::{new_alias, setup_and_load_program};

const EXAMPLE_PROGRAM: &str = r#"
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

fun String say_hello(Person p) {}

// VARIABLES:
//    Worker worker
//    Person alice
//    Person? bob
//    [String] cli_args
"#;

fn setup_checker<'a>(use_scope: bool, symbols_info: &'a GlobalSymbolsInfo) -> ExprTypeChecker<'a> {
    let scope = if use_scope {
        Some("Person".into())
    } else {
        None
    };
    let mut checker = ExprTypeChecker::new(symbols_info, new_alias("main"), scope);

    let person_type = Type::IdentQualified(new_alias("main"), "Person".into());

    checker.add_variable("alice".into(), person_type.clone()).unwrap();
    checker
        .add_variable("bob".into(), Type::Maybe(Box::new(person_type)))
        .unwrap();
    checker
        .add_variable("cli_args".into(), Type::List(Box::new(Type::String)))
        .unwrap();
    checker
        .add_variable(
            "worker".into(),
            Type::IdentQualified(new_alias("main"), "Worker".into()),
        )
        .unwrap();
    checker
}

fn parse_expr(expr_string: &str) -> Expr {
    let tokens = scan_tokens(&expr_string.into());
    let mut parser = Parser::create(tokens.expect("Scanning failed!"));
    parser.parse_expr().expect("Parsing failed!")
}

fn assert_expr_ok(expr_str: &str, use_scope: bool, expected_type: Type) {
    let mut original_expr = parse_expr(expr_str);
    let mut wp = setup_and_load_program(EXAMPLE_PROGRAM);
    let info = check_and_annotate_symbols(&mut wp).unwrap();
    let checker = setup_checker(use_scope, &info);

    let res = checker.calculate_and_annotate(&mut original_expr);
    assert!(res.is_ok(), "Typecheck failed: {}", res.unwrap_err());
    assert_eq!(res.unwrap(), expected_type);

    if let Expr::TypedExpr { expr: _, typename } = original_expr {
        assert_eq!(typename, expected_type);
    } else {
        assert!(false, "Not typed expression, but {:?}", original_expr);
    }
}

fn assert_expr_fails(expr_str: &str, use_scope: bool) {
    let mut expr = parse_expr(expr_str);
    let mut wp = setup_and_load_program(EXAMPLE_PROGRAM);
    let info = check_and_annotate_symbols(&mut wp).unwrap();
    let checker = setup_checker(use_scope, &info);

    let res = checker.calculate_and_annotate(&mut expr);
    assert!(
        res.is_err(),
        "Typecheck HAD TO FAIL, BUT resulted in : {:?}",
        res.unwrap()
    );
}

#[test]
fn test_simple_operator() {
    assert_expr_ok("1 + 2", false, Type::Int);
    assert_expr_ok("2.0 + 0.0", false, Type::Float);

    assert_expr_fails("2.0 + \"hello\" ", false);
}

// NO SCOPE
// 1+1 -> Int
// 1.0 + 1.0 -> Float
// [1, 2] + [1, 2] -> [Int]
// [2, 2] + ["asd"]  ERROR
// "asd" + "asd" String
