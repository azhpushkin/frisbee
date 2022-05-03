use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_is_fine!(
    assign_value_to_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() {
        String? name = "hello";
    }
    "#
);

assert_semantic_check_is_fine!(
    assign_nil_to_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() {
        String? name = nil;
    }
    "#
);

assert_semantic_check_is_fine!(
    return_nil_or_value_as_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int? other() { return nil; }
    fun Int? other() { return 2; }
    "#
);

assert_semantic_check_fails!(
    cant_assign_to_maybe_as_a_tuple,
    r#"
    ===== file: main.frisbee
    fun void main() {  // ERR: qwe
        String? name = (true, "asd");
    }
    "#
);

assert_semantic_check_fails!(
    cant_assign_to_maybe_using_index,
    r#"
    ===== file: main.frisbee
    fun void main() {  // ERR: qwe
        String? name[1] = "value";
    }
    "#
);

assert_semantic_check_fails!(
    assign_maybe_to_a_general_type,
    r#"
    ===== file: main.frisbee

    fun void main() {
        Int? a = 1;
        Int b = a;  // ERR: sorry but no..
    }
    "#
);

assert_semantic_check_is_fine!(
    cant_use_maybe_as_bool_operator,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Bool? a = true;
        a and true;  // ERR: no way sorry
    }
    "#
);

assert_semantic_check_is_fine!(
    cant_use_maybe_methods,
    r#"
    ===== file: main.frisbee
    fun void main() {
        String? a = "Hello";
        a.len();  // ERR: no way sorry
    }
    "#
);

assert_semantic_check_is_fine!(
    cant_use_maybe_in_ints_operators,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? a = 1;
        a + a;    // ERR: no way sorry
    }
    "#
);

assert_semantic_check_is_fine!(
    cant_use_maybe_as_list,
    r#"
    ===== file: main.frisbee
    fun void main() {
        [Int]? a = [];
        a + [1];
    }
    "#
);

assert_semantic_check_is_fine!(
    compare_maybe_to_a_nil,
    r#"
    ===== file: main.frisbee
    fun void main() { 
        Int? index = 4;
        index == nil;
    }
    "#
);

assert_semantic_check_is_fine!(
    compare_maybe_to_a_value,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? index = nil;
        index == 3;
    }
    "#
);

assert_semantic_check_fails!(
    cant_compare_maybe_to_another_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() { 
        Int? i = nil;
        Float f = nil;
        i == f;  // ERR: sorry cant do that!
    }
    "#
);

assert_semantic_check_is_fine!(
    compare_two_maybes,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int a? = 1;
        Int b? = nil;
        a == b;
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_inside_of_tuple_or_as_a_tuple,
    r#"
    ===== file: main.frisbee
    fun void main() {
        (Int, String?) a = (1, nil);
        (Int, String)? b = nil;
        (Int?, String)? c = (1, "asd");

        a[1] = 123;
        b = (1, "qwe");
        c = nil;
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_in_a_class,
    r#"
    ===== file: main.frisbee
    class Person {
        String name?;

        fun Person() {
            @name = nil;
        }

        fun set_name(String a) {
            @name = "asd";
        }

    }
    fun void main() {
        Person? p = nil;
        p = Person();
        p.set_name("Anton");
        p.name = "Tony";
        p.name == nil;
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_as_a_function_argument,
    r#"
    ===== file: main.frisbee
    fun Bool? is_hi(String? a) {
        if a == "" {
            return nil;
        }
        return a == "hi";
    }

    fun void main() {
        is_hi(nil);
        is_hi("qwe");
        is_hi(nil) == nil;
        is_hi(nil) != false
    }
    "#
);
