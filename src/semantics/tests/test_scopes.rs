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
        a = 1;  // ERR: not inited 1
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
            if false { Int a; }  // ERR: already defined 2
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
        Int b = a;  // ERR: qwe
    }
    "#
);


assert_semantic_check_fails!(
    foreach_variable_not_accessible,
    r#"
    ===== file: main.frisbee
    fun void main() {
        foreach i in range(0, 4) {}
        Int last_index = i;  // ERR: `i` is not here dude sry
    }
    "#
);


assert_semantic_check_fails!(
    foreach_variable_is_in_scope,
    r#"
    ===== file: main.frisbee
    fun void main() {
        foreach i in range(0, 4) {
            Int i;  // ERR: already defined i sorry
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