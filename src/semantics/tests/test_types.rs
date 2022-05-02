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
