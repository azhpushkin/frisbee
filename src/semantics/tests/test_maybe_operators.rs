use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_is_fine!(
    simple_elvis_usage,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? index = nil;
        Int i = index ?: 0;

        Int? another = index ?: 3;
    }
    "#
);

assert_semantic_check_is_fine!(
    elvis_with_tuple,
    r#"
    ===== file: main.frisbee
    fun void main() {
        (Int, String)? index = nil;
        (Int, String) i = index ?: (0, "");

        Int q = (index?:(0, ""))[0];
    }
    "#
);

assert_semantic_check_fails!(
    elvis_and_index_access_order,
    r#"
    ===== file: main.frisbee
    fun void main() {
        (Int, String)? index = nil;

        Int q = index?:(0, "")[0];  // ERR: Expected type `(Int, String)` but got `Int`
    }
    "#
);

assert_semantic_check_is_fine!(
    elvis_with_list,
    r#"
    ===== file: main.frisbee
    fun void main() {
        [Bool]? data = [];
        Bool b = (data ?: [true])[-1];

        data = nil;
        [Bool] b2 = data ?: [];
        
    }
    "#
);

assert_semantic_check_is_fine!(
    call_methods_on_elvis,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? index = nil;
        String q = (index ?: -1).to_string();
    }
    "#
);

assert_semantic_check_fails!(
    elvis_operator_precedence,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? index = nil;
        String q = index?:1  .to_string();  // ERR: Expected type `Int` but got `String`
    }
    "#
);

assert_semantic_check_fails!(
    elvis_wrong_right_side,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? index = nil;
        index ?: -1.0;  // ERR: Expected type `Int` but got `Float`
    }
    "#
);

assert_semantic_check_fails!(
    elvis_nil_in_right_side,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? index = nil;
        index ?: nil;  // ERR: `nil` is only allowed for maybe types (expected `Int`)
    }
    "#
);

assert_semantic_check_fails!(
    elvis_on_non_maybe,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int i = 0;
        i ?: -1;  // ERR: Maybe type must be left part of elvis, but got `Int`
    }
    "#
);

assert_semantic_check_is_fine!(
    elvis_complex_tuple_index,
    r#"
    ===== file: main.frisbee
    fun void main() {
        (String, Int?)? person = nil;

        (String, Int?) me = person ?: ("Anton", nil);
        me = person ?: ("Anton", 123);

        me = person ?: ("Anton", me[1]?:0);
    }
    "#
);

assert_semantic_check_fails!(
    normal_method_call_on_maybe_not_allowed,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? i = nil;
        i.to_string();  // ERR: Use ?. operator to access methods for Maybe type
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_method_call_simple,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? i = nil;
        String? s = i?.to_string();
        Int? a = i?.abs();
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_method_call_and_elvis,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? i = nil;
        String s = i?.to_string() ?: "nil";
        
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_elvis_method_call,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? i = nil;
        String s = (i?:-1).to_string();
    }
    "#
);

assert_semantic_check_fails!(
    maybe_method_on_non_maybe_fails,
    r#"
    ===== file: main.frisbee
    fun void main() {
        1?.to_string();  // ERR: ?. operator to can only be used on Maybe types, got Int
    }
    "#
);

assert_semantic_check_fails!(
    maybe_method_on_nil_fails,
    r#"
    ===== file: main.frisbee
    fun void main() {
        nil?.to_string();  // ERR: `nil` is not allowed here (cannot derive type)
    }
    "#
);

assert_semantic_check_is_fine!(
    maybe_method_on_custom_type,
    r#"
    ===== file: main.frisbee
    class Data {
        Int x; Int y;
        fun Int get_x() { return @x; }
        fun Int? get_y() { return @y; }
    }
    fun void main() {
        Data? d = Data(1, 2);

        Int? x = d?.get_x();
        Int? y = d?.get_y();
    }
    "#
);

assert_semantic_check_fails!(
    maybe_method_on_void_function_is_still_void,
    r#"
    ===== file: main.frisbee
    class Data {
        fun void lol() {}
    }
    fun void main() {
        Data? d = Data();
        d?.lol();
        Bool? q = d?.lol();  // ERR: Expected type `Bool?` but got `()?`
    }
    "#
);
