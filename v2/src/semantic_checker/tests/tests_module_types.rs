use crate::semantic_checker;
use crate::test_utils::TestFilesCreator;

#[test]
#[should_panic]
pub fn check_import_function_name_collision() {
    let mut t = TestFilesCreator::new();
    t.set_mainfile(
        r#"
        from mod import somefun;

        fun Nil somefun() {}
        "#,
    );
    t.set_file("mod.frisbee", "");

    semantic_checker::perform_checks(&t.load_program());
}

#[test]
#[should_panic]
pub fn check_import_active_type_name_collision() {
    let mut t = TestFilesCreator::new();
    t.set_mainfile(
        r#"
        from mod import Type;

        active Type {}
        "#,
    );
    t.set_file("mod.frisbee", "");

    semantic_checker::perform_checks(&t.load_program());
}

#[test]
#[should_panic]
pub fn check_no_self_referrings_for_active() {
    let mut t = TestFilesCreator::new();
    t.set_mainfile("active Type { Type type; }");

    semantic_checker::perform_checks(&t.load_program());
}

#[test]
#[should_panic]
pub fn check_no_self_referrings_for_passive() {
    let mut t = TestFilesCreator::new();
    t.set_mainfile("class Type { Type type; }");

    semantic_checker::perform_checks(&t.load_program());
}

#[test]
#[should_panic]
pub fn check_no_self_referrings_for_tuple() {
    let mut t = TestFilesCreator::new();
    t.set_mainfile("class Type { (Type, Int) type; }");

    semantic_checker::perform_checks(&t.load_program());
}
