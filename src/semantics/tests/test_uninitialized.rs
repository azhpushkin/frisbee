use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_fails!(
    simple_case_of_uninitialized,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int b;
        b = b + 1;  // ERR: Variable `b` might be uninitialized here
    }
    "#
);

assert_semantic_check_fails!(
    cant_assign_to_index_of_uninitialized_tuple,
    r#"
    ===== file: main.frisbee
    fun void main() {
        (Int, Int) a;
        a[0] = 1;  // ERR: Variable `a` might be uninitialized here
        a[1] = 2;
    }
    "#
);

assert_semantic_check_fails!(
    cant_return_initialized,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int b;
        return b;  // ERR: Variable `b` might be uninitialized here
    }
    "#
);

assert_semantic_check_fails!(
    uninitialized_after_branching,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        
        if true { a = 1; }
        else { }  // nothing here, so assume a can be uninitialized

        a + 1;  // ERR: Variable `a` might be uninitialized here
    }
    "#
);

assert_semantic_check_fails!(
    uninitialized_after_branching_with_elif,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        if true { a = 1; }
        elif true { a = 2; }
        else { }  // nothing here, so assume a can be uninitialized

        a + 1;  // ERR: Variable `a` might be uninitialized here
    }
    "#
);

assert_semantic_check_fails!(
    // if might never be executed, so do not only judge `a` based on them
    still_uninitialized_after_single_if_when_no_else,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        if true { a = 1; }
        a + 1;  // ERR: Variable `a` might be uninitialized here
    }
    "#
);

assert_semantic_check_fails!(
    // if-elif might never be executed, so do not only judge `a` based on them
    still_uninitialized_after_if_elif_when_no_else,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        if true { a = 1; }
        elif false { a = 2; }
        a + 1;  // ERR: Variable `a` might be uninitialized here
    }
    "#
);

assert_semantic_check_is_fine!(
    initialized_after_single_if_elif_else,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        
        if false { a = 1; }
        elif false {a = 2; }
        else { a = 3; }
        
        a + 1;
    }
    "#
);

assert_semantic_check_fails!(
    still_unitialized_after_while,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        while false { a = 1; a + 1;}
        a + 2;  // ERR: Variable `a` might be uninitialized here
    }
    "#
);

assert_semantic_check_fails!(
    uninitialized_if_continue_occures,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        while false {
            if true {
                continue;
                a = 1;
            } else {
                continue;
                a = 2;
            }
            a + 3;  // ERR: Variable `a` might be uninitialized here
        }
    }
    "#
);

assert_semantic_check_fails!(
    uninitialized_if_break_occures,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        while true {
            if true {
                a = 1;
            } else {
                // so from the code it is obvious that `a` is always set
                // but the fact that there is possible break must make
                // it impossible to know if `a` is initialized or not
                if false {
                    if true { }
                    else { break; }
                }
                a = 2;
            }
            a + 3;  // ERR: Variable `a` might be uninitialized here
        }
    }
    "#
);

assert_semantic_check_is_fine!(
    can_beinitialized_if_break_occures,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a;
        while true {
            if false { break; }
            a = 1;
            a + 1;
        }
    }
    "#
);

assert_semantic_check_is_fine!(
    constructor_that_sets_all_fields,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    class Person {
        String name; Int age;
        
        fun Person() {
            @name = "";
            @age = 0;
            println(@name + @age.to_string());
        }
    }
    "#
);

assert_semantic_check_fails!(
    uninitialized_field_in_constructor,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    class Person {
        String name; Int age;
        
        fun Person() {  // ERR: Constructor does not initialize field `age`
            @name = "";
        }
    }
    "#
);

assert_semantic_check_fails!(
    maybe_type_must_be_initialized_as_well,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    class Person {
        String? name; Int age;
        
        fun Person() {  // ERR: Constructor does not initialize field `name`
            @age = 0;
        }
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_type_might_be_inited_with_null,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    class Person {
        String? name; Int age;
        
        fun Person() {
            @age = 0;
            @name = nil;
        }
    }
    "#
);

assert_semantic_check_fails!(
    cant_access_own_before_initializing,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    class Person {
        String name; Int age;
        
        fun Person() {
            @age = 0;
            if true { @name = "Hello!"; }
            print(@name);  // ERR: Own field `name` cannot be used before initializing
        }
    }
    "#
);

assert_semantic_check_is_fine!(
    branching_implemented_for_own_fields,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    class Person {
        String name; Int age;
        
        fun Person() {
            @age = 0;
            if true { @name = "Hello!"; }
            else {@name = "Hi!"; }
            print(@name);
        }
    }
    "#
);
