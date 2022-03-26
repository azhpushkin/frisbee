use crate::semantic_checker::module_types;
use crate::test_utils::TestFilesCreator;

use crate::loader::*;

#[test]
#[should_panic]
pub fn check_import_function_name_collision() {
    let mut t = TestFilesCreator::new();
    t.add_mainfile(
        r#"
        from mod import somefun;

        fun Nil somefun() {}
        "#,
    );
    t.add_file("mod.frisbee", "");

    let wp = load_program(t.get_main_path()).unwrap();
    module_types::check_collision_of_imports_and_definitions(&wp);
}

#[test]
#[should_panic]
pub fn check_import_active_type_name_collision() {
    let mut t = TestFilesCreator::new();
    t.add_mainfile(
        r#"
        from mod import Type;

        active Type {}
        "#,
    );
    t.add_file("mod.frisbee", "");

    let wp = load_program(t.get_main_path()).unwrap();
    module_types::check_collision_of_imports_and_definitions(&wp);
}
