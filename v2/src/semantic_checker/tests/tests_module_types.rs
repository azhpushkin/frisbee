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

    assert!(semantic_checker::perform_checks(&wp).is_some());
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

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
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

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
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

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
pub fn check_active_and_class_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {}
        active Type {}
    "#,
    );

    semantic_checker::perform_checks(&wp);
}

#[test]
pub fn check_self_referrings_for_active_are_allowed() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        active Type { Type type; }
    "#,
    );

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
pub fn check_no_self_referrings_for_passive() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { Type type; }
    "#,
    );

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
pub fn check_no_self_referrings_for_tuple() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { (Type, Int) type; }
    "#,
    );

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
pub fn check_no_self_referrings_in_imports() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
pub fn check_imported_typess_are_existing() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import X1;
        ===== file: module.frisbee
        class X {}
    "#,
    );

    semantic_checker::perform_checks(&wp);
}

#[test]
#[should_panic]
pub fn check_imported_functions_are_existing() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import func;
        ===== file: module.frisbee
            // empty file
    "#,
    );

    semantic_checker::perform_checks(&wp);
}
