use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_fails!(
    return_type_matches,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun String other() {
        return 1 + 1;  // ERR: Expected type `String` but got `Int`
    }
    "#
);

assert_semantic_check_fails!(
    add_float_and_int,
    r#"
    ===== file: main.frisbee
    fun void main() {
        1 + 1.0;  // ERR: Cannot apply Plus to `Int` and `Float`
    }
    "#
);

assert_semantic_check_is_fine!(
    chained_bool_operators,
    r#"
    ===== file: main.frisbee
    fun void main() {
        true and not false or not (1 == 2);
    }
    "#
);

assert_semantic_check_is_fine!(
    chained_int_operators,
    r#"
    ===== file: main.frisbee
    fun void main() {
        1 * 2 - 3 --5 / 0;
    }
    "#
);

assert_semantic_check_is_fine!(
    access_tuple_inside_list,
    r#"
    ===== file: main.frisbee
    fun void main() {
        [(String, Int)] list = [("", 0)];
        list[0][0] = "Hello";
        list[0][1] = 1;

        (String, Int) a = list[1];
    }
    "#
);
