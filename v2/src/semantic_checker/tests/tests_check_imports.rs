use crate::semantic_checker::module_types::check_module_types;
use crate::test_utils::TestFilesCreator;

use crate::loader::*;
use tempfile::{tempdir, tempfile};

#[test]
#[should_panic]
pub fn check_import_name_collision() {
    let mut t = TestFilesCreator::new();
    t.add_mainfile(
        r#"
        from mod import somefun;

        fun Nil somefun() {}
        "#,
    );
    t.add_file("mod.frisbee", "");

    let wp = load_program(t.get_main_path()).unwrap();
    check_module_types(&wp);
}
