use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_fails!(
    main_must_return_void,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int other() {  // ERR: Function `other` is not guaranteed to return a value
        Int a = 1;
    }
    "#
);

assert_semantic_check_is_fine!(
    void_function_can_have_no_returns,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun void other() {
        if true {return;} else {}
    }
    "#
);

assert_semantic_check_fails!(
    return_in_a_while_not_enough,
    r#"
    ===== file: main.frisbee
    fun void main() {}
    fun Int other() {  // ERR: Function `other` is not guaranteed to return a value
        while true {
            return 1;
        }
    }
    "#
);

assert_semantic_check_fails!(
    partial_return_in_if_is_not_enough,
    r#"
    ===== file: main.frisbee
    fun void main() {}
    fun Int other() {  // ERR: Function `other` is not guaranteed to return a value
        if true {
            return 1;
        } else {}
    }
    "#
);

assert_semantic_check_is_fine!(
    all_branches_must_have_return,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int other() {
        Bool v = false;
        if v == true {
            return 1;
        } elif false {
            return 2;
        } else {
            return 3;
        }
    }
    "#
);

assert_semantic_check_fails!(
    all_branches_must_have_return_or_err,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int other() {  // ERR: Function `other` is not guaranteed to return a value
        Bool v = false;
        if v == true {
            return 1 ;
        } elif true {
            // O_o
        } else {
            return 2;
        }
    }
    "#
);

assert_semantic_check_fails!(
    while_loop_with_break_must_have_return,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int other() {  // ERR: Function `other` is not guaranteed to return a value
        while true {
            if false {
                break;
            }
            
            return 2;
        }
    }
    "#
);


assert_semantic_check_fails!(
    while_loop_with_continue_must_have_return,
    r#"
    ===== file: main.frisbee
    fun void main() {}

    fun Int other() {  // ERR: Function `other` is not guaranteed to return a value
        while true {
            if false {
                return 1;
            } else {
                continue;
                return 3;
            }
            
            return 2;
        }
    }
    "#
);
