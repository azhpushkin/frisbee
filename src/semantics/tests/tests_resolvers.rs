use crate::semantics::resolvers::NameResolver;
use crate::test_utils::setup_and_load_program;

#[test]
pub fn check_import_from_same_module_is_fine() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import Type;

        ===== file: mod.frisbee
        fun Nil somefun() {}
        class Type {}
    "#,
    );

    NameResolver::create(&mut wp).unwrap();
}

#[test]
pub fn check_import_of_same_function_are_not_allowed() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import somefun;

        
        ===== file: mod.frisbee
        fun Nil somefun() {}
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}

#[test]
pub fn check_import_function_name_collision() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;

        fun Nil somefun() {}
        ===== file: mod.frisbee
        fun Bool somefun() {}
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}

#[test]
pub fn check_import_active_type_name_collision() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import Type;

        active Type {}
        ===== file: mod.frisbee
          // empty file
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}

#[test]
pub fn check_active_and_class_name_collision() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {}
        active Type {}
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}

// #[test]
pub fn check_method_name_collisions() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {
            fun Nil hello() {}
            fun Nil hello() {}
        }
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}

#[test]
pub fn check_same_function_names_are_fine() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import hello, Person;

        fun Nil samename(Person someone) {}
        ===== file: mod.frisbee
        fun Person samename() {}
        fun Nil hello() {}

        class Person {}
    "#,
    );
    NameResolver::create(&mut wp).unwrap();
}

#[test]
pub fn check_no_self_referrings_in_imports() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}

#[test]
pub fn check_imported_types_are_existing() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import X1;
        ===== file: module.frisbee
        class X {}
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}

#[test]
pub fn check_imported_functions_are_existing() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import func;
        ===== file: module.frisbee
            // empty file
    "#,
    );

    assert!(NameResolver::create(&mut wp).is_err());
}
