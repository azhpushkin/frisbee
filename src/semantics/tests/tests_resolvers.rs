use crate::semantics::resolvers::NameResolver;
use crate::tests::helpers::setup_and_load_program;

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

    let modules_with_ast: Vec<_> = wp.iter().collect();
    NameResolver::create(&modules_with_ast).unwrap();
}

#[test]
pub fn check_import_of_same_function_are_not_allowed() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import somefun;

        
        ===== file: mod.frisbee
        fun Nil somefun() {}
    "#,
    );

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
}

#[test]
pub fn check_import_function_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;

        fun Nil somefun() {}
        ===== file: mod.frisbee
        fun Bool somefun() {}
    "#,
    );

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
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

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
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

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
}

// #[test]
pub fn check_method_name_collisions() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {
            fun Nil hello() {}
            fun Nil hello() {}
        }
    "#,
    );

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
}

#[test]
pub fn check_same_function_names_are_fine() {
    let wp = setup_and_load_program(
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
    let modules_with_ast: Vec<_> = wp.iter().collect();
    NameResolver::create(&modules_with_ast).unwrap();
}

#[test]
pub fn check_no_self_referrings_in_imports() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
}

#[test]
pub fn check_imported_types_are_existing() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import X1;
        ===== file: module.frisbee
        class X {}
    "#,
    );

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
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

    let modules_with_ast: Vec<_> = wp.iter().collect();
    assert!(NameResolver::create(&modules_with_ast).is_err());
}
