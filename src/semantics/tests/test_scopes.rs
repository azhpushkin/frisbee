use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_is_fine!(
    different_scopes_in_if_else_branches,
    r#"
    ===== file: main.frisbee
    fun void main() {
        if true { Int a; } 
        elif true { (String, Bool) a; }
        else { [Float] a; }
    }
    "#
);

assert_semantic_check_fails!(
    scope_dropped_after_if_else,
    r#"
    ===== file: main.frisbee
    fun void main() {
        if true { Int a = 4; } 
        else { Int a = 8; }
        a = 1;  // ERR: Variable `a` not defined
    }
    "#
);

assert_semantic_check_fails!(
    scope_passed_to_inner_if_else,
    r#"
    ===== file: main.frisbee
    fun void main() {
        if true { 
            Int a;
            if false { Int a; }  // ERR: Variable `a` was already defined before
        } 
    }
    "#
);

assert_semantic_check_fails!(
    scope_dropped_after_while,
    r#"
    ===== file: main.frisbee
    fun void main() {
        while true { Int a = 0; }
        Int b = a;  // ERR: Variable `a` not defined
    }
    "#
);

assert_semantic_check_fails!(
    foreach_variable_not_accessible,
    r#"
    ===== file: main.frisbee
    fun void main() {
        foreach i in range(0, 4) {}
        Int last_index = i;  // ERR: Variable `i` not defined
    }
    "#
);

assert_semantic_check_fails!(
    foreach_variable_is_in_scope,
    r#"
    ===== file: main.frisbee
    fun void main() {
        foreach i in range(0, 4) {
            Int i;  // ERR: Variable `i` was already defined before
        }
    }
    "#
);

assert_semantic_check_is_fine!(
    foreach_scope_is_dropped,
    r#"
    ===== file: main.frisbee
    fun void main() {
        foreach i in range(0, 4) {
            Int x = 2;
        }
        Int i;
        String x;
    }
    "#
);


assert_semantic_check_is_fine!(
    same_iter_in_multiple_foreach,
    r#"
    ===== file: main.frisbee
    fun void main() {
        foreach i in range(0, 4) {
            print((i + 1).to_string());
        }

        foreach s in ["a", "b", "c"] {
            print(s);
        }
    }
    "#
);
