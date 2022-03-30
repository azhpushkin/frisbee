use crate::semantic_checker;
use crate::test_utils::setup_and_load_program;

#[test]
pub fn check_import_from_same_module_is_fine() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import Type;

        ===== file: mod.frisbee
        fun Nil somefun() {}
        class Type {}
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_ok());
}

#[test]
pub fn check_import_of_same_obj_are_not_allowed() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import somefun;

        
        ===== file: mod.frisbee
        fun Nil somefun() {}
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_import_function_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;

        fun Nil somefun() {}
        ===== file: mod.frisbee
        fun Bool somefun()
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_import_active_type_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import Type;

        active Type {}
        ===== file: mod.frisbee
          // empty file
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_active_and_class_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {}
        active Type {}
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_self_referrings_for_active_are_allowed() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        active Type { Type type; }
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_ok());
}

#[test]
pub fn check_no_self_referrings_for_passive() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { Type type; }
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_no_self_referrings_for_tuple() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { (Type, Int) type; }
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_no_self_referrings_in_imports() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_imported_typess_are_existing() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import X1;
        ===== file: module.frisbee
        class X {}
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}

#[test]
pub fn check_imported_functions_are_existing() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import func;
        ===== file: module.frisbee
            // empty file
    "#,
    );

    assert!(semantic_checker::perform_checks(&wp).is_err());
}
