use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_is_fine!(
    assign_value_to_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() {
        (String, Bool)? name = ("hello", false);
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

assert_semantic_check_fails!(
    cant_assign_nil_to_non_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Bool name = nil;  // ERR: `nil` is only allowed for maybe types (expected `Bool`)
    }
    "#
);

assert_semantic_check_is_fine!(
    return_nil_or_value_as_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int? other1() { return nil; }
    fun Int? other2() { return 2; }
    "#
);

assert_semantic_check_fails!(
    cant_assign_to_maybe_as_a_tuple,
    r#"
    ===== file: main.frisbee
    fun void main() {  
        String? name = (true, "asd");  // ERR: Unexpected tuple value (expected `String?`)
    }
    "#
);

assert_semantic_check_fails!(
    cant_assign_to_maybe_using_index,
    r#"
    ===== file: main.frisbee
    fun void main() {  
        String? name = "";
        name[1] = "value";  // ERR: Only lists and tuples implement index access (got `String?`)
    }
    "#
);

assert_semantic_check_fails!(
    assign_maybe_to_a_general_type,
    r#"
    ===== file: main.frisbee

    fun void main() {
        Int? a = 1;
        Int b = a;  // ERR: Expected type `Int` but got `Int?`
    }
    "#
);

assert_semantic_check_fails!(
    cant_use_maybe_as_bool_operator,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Bool? a = true;
        a and true;  // ERR: Cannot apply And to `Bool?` and `Bool`
    }
    "#
);

assert_semantic_check_fails!(
    cant_use_maybe_methods,
    r#"
    ===== file: main.frisbee
    fun void main() {
        String? a = "Hello";
        a.len();  // ERR: Use ?. operator to access methods for Maybe type
    }
    "#
);

assert_semantic_check_fails!(
    cant_use_maybe_in_ints_operators,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? a = 1;
        a + a;    // ERR: Cannot apply Plus to `Int?` and `Int?`
    }
    "#
);

assert_semantic_check_fails!(
    cant_use_maybe_as_list,
    r#"
    ===== file: main.frisbee
    fun void main() {
        [Int]? a = [];
        a + [1];  // ERR: Cannot apply Plus to `[Int]?` and `[Int]`
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
        Float? f = nil;
        i == f;  // ERR: Types `Int?` and `Float?` cannot be checked for equality
    }
    "#
);

assert_semantic_check_is_fine!(
    compare_two_maybes,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? a = 1;
        Int? b = nil;
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

        a[1] = "inner";
        a = (1, nil);
        b = (1, "qwe");
        c = nil;
        c = (nil, "str");
    }
    "#
);

assert_semantic_check_fails!(
    cant_assign_inside_maybe_tuple,
    r#"
    ===== file: main.frisbee
    fun void main() {
        (Int, String)? b = nil;
        b[0] = 123;  // ERR: Only lists and tuples implement index access (got `(Int, String)?`)
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_in_a_class,
    r#"
    ===== file: main.frisbee
    class Person {
        String? name;

        fun Person() {
            @name = nil;
        }

        fun void set_name(String a) {
            @name = "asd";
        }

    }
    fun void main() {
        Person? p = nil;
        p = Person();

        Person p2 = Person();
        p2.set_name("Anton");
        p2.name = "Tony";
        p2.name == nil;
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
        is_hi(nil) != false;
    }
    "#
);
