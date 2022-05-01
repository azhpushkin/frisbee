use super::helpers::assert_semantic_check_fails;

assert_semantic_check_fails!(
    main_must_return_voide,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int other() {  // ERR: Function `other` is not guaranteed to return a value
        Int a = 1;
    }
    "#
);
