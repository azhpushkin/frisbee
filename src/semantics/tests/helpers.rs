use regex::Regex;

use crate::alias::ModuleAlias;
use crate::errors::{adjust_error_window, CompileError};
use crate::loader::WholeProgram;
use crate::tests::helpers::setup_and_load_program;

fn extract_expected_err<'a>(wp: &'a WholeProgram) -> (&'a ModuleAlias, usize, &'a str) {
    let expected_error_regex = Regex::new(r"// ERR: (?P<errmsg>.+)").unwrap();

    for (module_alias, file) in wp.files.iter() {
        for (i, line) in file.contents.split('\n').enumerate() {
            if let Some(re_match) = expected_error_regex.captures(line) {
                let errmsg = re_match.name("errmsg").unwrap().as_str();
                return (module_alias, i, errmsg);
            }
        }
    }
    unreachable!("No expected error found");
}

pub(super) fn run_semantic_and_check_error(program: &str) {
    let mut wp = setup_and_load_program(program);
    let semantic_res = crate::loader::check_and_aggregate(&mut wp);

    assert!(semantic_res.is_err(), "Expected error but got success");

    let semantic_err = semantic_res.unwrap_err();
    let expected_err = extract_expected_err(&wp);

    let (start, end) = semantic_err.error.get_position_window();
    let error_window = adjust_error_window(&wp.files[expected_err.0].contents, start, end);

    assert_eq!(&semantic_err.module, expected_err.0);
    assert_eq!(error_window.line, expected_err.1);
    assert_eq!(semantic_err.error.get_message(), expected_err.2);
}

macro_rules! assert_semantic_check_fails {
    ($name:ident, $program:literal) => {
        #[test]
        fn $name() {
            crate::semantics::tests::helpers::run_semantic_and_check_error($program);
        }
    };
}
pub(super) use assert_semantic_check_fails;
