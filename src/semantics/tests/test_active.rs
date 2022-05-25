use super::helpers::{assert_semantic_check_fails, assert_semantic_check_is_fine};

assert_semantic_check_fails!(
    active_must_be_spawned,
    r#"
    ===== file: main.frisbee
    active Actor {}

    fun void main() {
        Actor a = spawn Actor();
        Actor b = Actor();  // ERR: Active objects must be spawned, but `main::Actor` is created as a passive object
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
        Actor a = spawn Actor(0);
        a.counter;  // ERR: Can't access fields of active objects (only accessible from inside as @counter)
    }
    "#
);

assert_semantic_check_fails!(
    active_fields_cant_be_used_via_this,
    r#"
    ===== file: main.frisbee
    active Actor {
        Int counter;

        fun void reset() {
            this.counter = 0;  // ERR: Can't access fields of active objects (only accessible from inside as @counter)
        }
    }

    fun void main() {
        Actor a = spawn Actor(0);
        a ! reset();
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
        a.receiver();  // ERR: Can't call methods of active objects directly (use ! to send message or @receiver for access from inside)
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
        Actor root = spawn Actor(0, nil);
        
        Actor node = spawn Actor(1, spawn Actor(2, nil));
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
        
        fun Actor() {}  // ERR: Constructor does not initialize field `counter`
    }
    fun void main() {}
    "#
);

assert_semantic_check_fails!(
    active_constructor_cant_use_uninited,
    r#"
    ===== file: main.frisbee
    active Actor {
        Int counter;
        
        fun Actor() {
            @counter = @counter + 1;  // ERR: Own field `counter` cannot be used before initializing
        }  
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
    active_cant_both_call_and_send_to_own_methods,
    r#"
    ===== file: main.frisbee
    active Actor {
        fun Int get_counter(Int i) { 
            if i == 0 {
                return @get_counter(i + 1);
            }
            elif i == 1 {
                this ! get_counter(i + 1);
                return -1;
            }
            else {
                return i;
            }
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
        fun Int get_counter(Int i) {
            println("Got " + i.to_string());
            if i > 0 {
                send_msg(this, i - 1);
            }
            return i - 1;
        }
    }
    fun void send_msg(Actor a, Int i) {
        a ! get_counter(i);
    }

    fun void main() {
        Actor a = spawn Actor();
        send_msg(a, 5);
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
        Passive a = spawn Passive();  // ERR: Only active objects can be spawned, but `main::Passive` is not active
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
        a ! get_counter();    // ERR: Can only send message to active objects, but `main::Passive` is not active
    }
    "#
);
