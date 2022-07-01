use regex::Regex;

use crate::alias::ModuleAlias;
use crate::errors::{adjust_error_window, CompileError};
use crate::loader::WholeProgram;
use crate::semantics::aggregate::ProgramAggregate;
use crate::tests::helpers::setup_and_load_program;

fn extract_expected_err<'a>(wp: &'a WholeProgram) -> (&'a ModuleAlias, usize, &'a str) {
    let expected_error_regex = Regex::new(r"// ERR: (?P<errmsg>.+)").unwrap();

    for (module_alias, file) in wp.modules.iter() {
        for (i, line) in file.contents.split('\n').enumerate() {
            if let Some(re_match) = expected_error_regex.captures(line) {
                let errmsg = re_match.name("errmsg").unwrap().as_str();
                return (module_alias, i, errmsg);
            }
        }
    }
    unreachable!("Please check the test, no // ERR: found");
}

pub(super) fn run_semantic_and_check_error(program: &str) {
    let mut wp = setup_and_load_program(program);
    let semantic_res = crate::loader::check_and_aggregate(&mut wp);

    assert!(semantic_res.is_err(), "Expected error but got success");

    let semantic_err = semantic_res.unwrap_err();
    let (start, end) = semantic_err.error.get_position_window();
    let error_window = adjust_error_window(&wp.modules[&semantic_err.module].contents, start, end);

    let expected_err = extract_expected_err(&wp);
    assert_eq!(
        (
            &semantic_err.module,
            error_window.line,
            semantic_err.error.get_message().as_str()
        ),
        expected_err
    );
}

pub(super) fn run_semantic_and_check_all_good(program: &str) {
    let mut wp = crate::tests::helpers::setup_and_load_program(program);
    let res = crate::loader::check_and_aggregate(&mut wp);

    if res.is_err() {
        println!("{:?}", res);
    }
    assert!(res.is_ok());

    let res = res.unwrap();

    // BONUS: check that if the program is valid - it does not panic during codegen
    let ProgramAggregate { types, functions, entry } = res;
    let types: Vec<_> = types.into_iter().map(|(_, v)| v).collect();
    let functions: Vec<_> = functions.into_iter().map(|(_, v)| v).collect();
    crate::codegen::generate(&types, &functions, &entry);
}

macro_rules! assert_semantic_check_fails {
    ($name:ident, $program:literal) => {
        #[test]
        fn $name() {
            crate::semantics::tests::helpers::run_semantic_and_check_error($program);
        }
    };
}

macro_rules! assert_semantic_check_is_fine {
    ($name:ident, $program:literal) => {
        #[test]
        fn $name() {
            crate::semantics::tests::helpers::run_semantic_and_check_all_good($program);
        }
    };
}
pub(super) use assert_semantic_check_fails;
pub(super) use assert_semantic_check_is_fine;
