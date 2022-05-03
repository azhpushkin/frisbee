use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_fails!(
    return_type_matches,
    r#"
    ===== file: main.frisbee
    fun void main() {
        Int? a = 1;
        Int? b = 2;

        if (a != nil) {
            // a is fine here because it is confirmed
            Int c = b ?: a;
        }
    }

    "#
);
