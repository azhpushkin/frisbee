// use regex::Regex;

use crate::test_utils::setup_and_load_program;

pub(super) fn run_semantic_and_check_error(program: &str) {
    let mut wp = setup_and_load_program(program);
    crate::semantics::add_default_constructors(
        wp.files
            .iter_mut()
            .flat_map(|(_, loaded_file)| loaded_file.ast.types.iter_mut()),
    );
    let modules: Vec<_> = wp.iter().collect();
    crate::semantics::perform_semantic_analysis(&modules, &wp.main_module).unwrap();
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
