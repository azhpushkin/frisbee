use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_is_fine!(
    check_import_from_same_module_is_fine,
    r#"
    ===== file: main.frisbee
    from mod import somefun;
    from mod import Type;

    fun void main() {}
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
    fun Person samename() {
        Person p = Person();
        return p;
    }
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
    from module import X1;  // ERR: Imported type `X1` is not defined in module `module`!
    fun void main() {}

    ===== file: module.frisbee
    class X {}
    "#
);

assert_semantic_check_fails!(
    check_imported_functions_are_existing,
    r#"
    ===== file: main.frisbee
    from lol import func;  // ERR: Imported function `func` is not defined in module `lol`!
    fun void main() {}

    ===== file: lol.frisbee
        // empty file
    "#
);

assert_semantic_check_fails!(
    check_entry_function_must_be_defined_in_main_module,
    r#"
    ===== file: main.frisbee
    // ERR: Entry function `main` not found
    // ^^ this error always occurs on first line
    
    from mod import main;
    ===== file: mod.frisbee
    fun void main() {}
    "#
);

assert_semantic_check_fails!(
    check_entry_function_return_type,
    r#"
    ===== file: main.frisbee
    fun [Int] main() {  // ERR: Entry function `main` must return void, but it returns [Int]
        return [0];
    }
    "#
);
