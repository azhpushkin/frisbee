use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_is_fine!(
    check_import_from_same_module_is_fine,
    r#"
    ===== file: main.frisbee
    from mod import somefun;
    from mod import Type;

    ===== file: mod.frisbee
    fun void somefun() {}
    class Type {}
    "#
);

assert_semantic_check_fails!(
    check_import_of_same_function_are_not_allowed,
    r#"
    ===== file: main.frisbee
    from mod import somefun;
    from mod import somefun;  // ERR: Function `somefun` is already introduced in this module

    fun void main() {}
    ===== file: mod.frisbee
    fun void somefun() {}
    "#
);

assert_semantic_check_fails!(
    check_import_function_name_collision,
    r#"
    ===== file: main.frisbee
    from mod import somefun;  // ERR: Function `somefun` is already introduced in this module

    fun void somefun() {}
    fun void main() {}
    ===== file: mod.frisbee
    fun Bool somefun() {}
    "#
);

assert_semantic_check_fails!(
    check_import_active_type_name_collision,
    r#"
    ===== file: main.frisbee
    from mod import Type;  // ERR: Type `Type` is already introduced in this module

    active Type {}
    fun void main() {}
    ===== file: mod.frisbee
        // empty file
    "#
);

assert_semantic_check_fails!(
    check_active_and_class_name_collision,
    r#"
    ===== file: main.frisbee
    class Foo {}
    active Foo {}  // ERR: Type `Foo` is already introduced in this module

    fun void main() {}
    "#
);

assert_semantic_check_fails!(
    check_method_name_collisions,
    r#"
    ===== file: main.frisbee
    class Type {
        fun void hello() {}
        fun void hello() {}  // ERR: Method `hello` defined more than once in `Type`
    }

    fun void main() {}
    "#
);

assert_semantic_check_is_fine!(
    check_same_function_names_are_fine,
    r#"
    ===== file: main.frisbee
    from mod import hello, Person;

    fun void samename(Person someone) {}
    fun void main() {}
    ===== file: mod.frisbee
    fun Person samename() {}
    fun void hello() {}

    class Person {}
    "#
);

assert_semantic_check_fails!(
    check_no_self_referrings_in_imports,
    r#"
    ===== file: main.frisbee
    from main import Type;  // ERR: Self-imports are not allowed
    fun void main() {}
    "#
);

assert_semantic_check_fails!(
    check_imported_types_are_existing,
    r#"
    ===== file: main.frisbee
    from module import X1;
    fun void main() {}

    ===== file: module.frisbee// ERR: already defined
    class X {}
    "#
);

assert_semantic_check_fails!(
    check_imported_functions_are_existing,
    r#"
    ===== file: main.frisbee// ERR: already defined
    from module import func;
    fun void main() {}

    ===== file: module.frisbee
        // empty file
    "#
);
