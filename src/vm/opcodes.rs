macro_rules! opcodes_list {
    ($s:expr, [$($acc:expr),*], $c:ident($args:literal)  ) => {
        pub const $c: u8 = $s;

        const OPCODES_INFO: [(u8, &str); $s+1] = [$($acc),*, ($args, stringify!($c))];
    };
    ($s:expr, [$($acc:expr),+], $c:ident($args:literal), $ ( $more:ident($margs:literal) ),+ $(,)? ) => {
        pub const $c: u8 = $s;

        opcodes_list!($s+1, [$($acc),*, ($args, stringify!($c))], $(  $more($margs) ),+ );
    };

    // Empty array call
    ($s:expr, [], $c:ident($args:literal), $ ( $more:ident($margs:literal) ),+ $(,)? ) => {
        pub const $c: u8 = $s;

        opcodes_list!($s+1, [($args, stringify!($c))], $(  $more($margs) ),+ );
    };

    // Initial call
    ($c:ident($args:literal), $ ( $more:ident($margs:literal) ),+ $(,)? ) => {
        opcodes_list!(0, [], $c($args), $(  $more($margs) ),+ );
    };
}

#[rustfmt::skip]
pub mod op {

    opcodes_list!(
        LOAD_TRUE(0),
        LOAD_FALSE(0),
        NEGATE_INT(0),
        ADD_INT(0),
        SUB_INT(0),
        MUL_INT(0),
        DIV_INT(0),
        GREATER_INT(0),
        LESS_INT(0),
        EQ_INT(0),
        NEGATE_FLOAT(0),
        ADD_FLOAT(0),
        SUB_FLOAT(0),
        MUL_FLOAT(0),
        DIV_FLOAT(0),
        GREATER_FLOAT(0),
        LESS_FLOAT(0),
        EQ_FLOAT(0),
        NEGATE_BOOL(0),
        EQ_BOOL(0),
        AND_BOOL(0),
        OR_BOOL(0),
        ADD_STRINGS(0),
        EQ_STRINGS(0),
        GET_LIST_ITEM(0),
        RETURN(0),

        ALLOCATE(1),

        RESERVE(1),
        POP(1),

        LOAD_CONST(1),
        LOAD_SMALL_INT(1),

        ALLOCATE_LIST(2),  // item_size + initial_size

        JUMP(2),
        JUMP_BACK(2),
        JUMP_IF_FALSE(2),

        SET_LOCAL(2),  // offset + size
        GET_LOCAL(2),  // offset + size

        SET_OBJ_FIELD(2),  // offset from pointer, size
        GET_OBJ_FIELD(2), // offset from pointer, size

        SET_LIST_ITEM(2),  // offset from pointer, size of value to set

        GET_TUPLE_ITEM(3),

        // return size, total offset, call position
        CALL(4),
        CALL_STD(4)
    );

    pub fn get_args_num(opcode: u8) -> usize {
        OPCODES_INFO[opcode as usize].0 as usize
    }

    pub fn get_display_name(opcode: u8) -> &'static str {
        OPCODES_INFO[opcode as usize].1
    }

    pub const CONST_END_FLAG: u8 = u8::MAX;
    pub const CONST_INT_FLAG: u8 = 1;    
    pub const CONST_FLOAT_FLAG: u8 = 2;
    pub const CONST_STRING_FLAG: u8 = 3;
}
