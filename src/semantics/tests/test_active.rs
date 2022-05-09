use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_fails!(
    active_must_be_spawned,
    r#"
    ===== file: main.frisbee
    active Actor {}

    fun void main() {
        Actor a = spawn Actor();
        Actor b = Actor();  // ERR: sorry no way
    }
    "#
);

assert_semantic_check_fails!(
    active_fields_cant_be_used,
    r#"
    ===== file: main.frisbee
    active Actor {
        Int counter;
    }

    fun void main() {
        Actor a = spawn Actor();
        a.counter;  // ERR: sorry no way
    }
    "#
);

assert_semantic_check_fails!(
    active_methods_cant_be_used,
    r#"
    ===== file: main.frisbee
    active Actor {
        fun void receiver() {}
    }

    fun void main() {
        Actor a = spawn Actor();
        a.receiver();  // ERR: sorry no way
    }
    "#
);

assert_semantic_check_is_fine!(
    active_can_use_own_methods_and_fields,
    r#"
    ===== file: main.frisbee
    active Actor {
        Int counter;

        fun Int get_counter() { return @counter; }
        fun void receiver() {
            @counter = @get_counter() + 1;
        }
    }

    fun void main() {}
    "#
);

assert_semantic_check_is_fine!(
    active_constructor_with_self_reference,
    r#"
    ===== file: main.frisbee
    active Actor {
        Int counter;
        Actor? parent;
    }

    fun void main() {
        Actor root = spawn Actor(0, null);
        
        Actor node = spawn Actor(1, spawn Actor(2, null));
        spawn Actor(3, root);
    }
    "#
);

assert_semantic_check_fails!(
    active_constructor_must_initialize_fields,
    r#"
    ===== file: main.frisbee
    active Actor {
        Int counter;
        
        fun Actor() {}  // ERR: constructor does not initialize all
    }
    fun void main() {}
    "#
);

assert_semantic_check_is_fine!(
    active_receivers_accessed_through_send,
    r#"
    ===== file: main.frisbee
    active Actor {
        fun Int get_counter() { return 1; }
    }
    fun void main() {
        Actor a = spawn Actor();
        a ! get_counter();
    }
    "#
);

assert_semantic_check_is_fine!(
    active_own_methods_both_call_and_send,
    r#"
    ===== file: main.frisbee
    active Actor {
        fun Int get_counter(Int i) { 
            if i == 0 { @get_counter(i + 1); }
            elif i == 1 { this ! get_counter(i + 1); }
            else { return i; }
         }
    }
    fun void main() {}
    "#
);

assert_semantic_check_is_fine!(
    active_pass_as_reference,
    r#"
    ===== file: main.frisbee
    active Actor {
        fun void get_counter(Int i) {
            if i == 1 {
                send_msg(this, i - 1);
            }
            return 0;
        }
    }
    fun send_msg(Actor a, Int i) {
        a ! get_counter(i);
    }

    fun void main() {
        Actor a = spawn Actor();
        send_msg(a, 0);
    }
    "#
);



assert_semantic_check_fails!(
    spawning_passive_not_allowed,
    r#"
    ===== file: main.frisbee
    class Passive {
        fun void get_counter() {}
    }

    fun void main() {
        Passive a = spawn Passive();  // ERR: not allowed
    }
    "#
);



assert_semantic_check_fails!(
    send_to_passive_not_allowed,
    r#"
    ===== file: main.frisbee
    class Passive {
        fun void get_counter() {}
    }

    fun void main() {
        Passive a = Passive();
        a ! get_counter();    // ERR: not allowed 1
    }
    "#
);
