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
        LOAD_CONST(1),  // constant index
        LOAD_SMALL_INT(1),  // small int value (0-255)

        // Integer operators
        NEGATE_INT(0),
        ADD_INT(0),
        SUB_INT(0),
        MUL_INT(0),
        DIV_INT(0),
        GREATER_INT(0),
        LESS_INT(0),
        EQ_INT(0),

        // Float operators
        NEGATE_FLOAT(0),
        ADD_FLOAT(0),
        SUB_FLOAT(0),
        MUL_FLOAT(0),
        DIV_FLOAT(0),
        GREATER_FLOAT(0),
        LESS_FLOAT(0),
        EQ_FLOAT(0),

        // Bool operators
        NEGATE_BOOL(0),
        EQ_BOOL(0),
        AND_BOOL(0),
        OR_BOOL(0),
        ADD_STRINGS(0),
        EQ_STRINGS(0),

        RESERVE(1),  // size to reserve
        POP(1),  // size to pop

        SET_LOCAL(2),  // offset + size
        GET_LOCAL(2),  // offset + size

        // Extract part of the tuple when whole tuple is on stack
        // args: total tuple size, offset to extract, size to extract
        GET_TUPLE_ITEM(3),

        JUMP(2),  // relative position as u16
        JUMP_BACK(2),  // relative position as u16
        JUMP_IF_FALSE(2),  // relative position as u16

        CALL(3),  // locals size, call position as u16
        CALL_STD(3),  // locals size, call position as u16
        RETURN(1),  // size of return value

        ALLOCATE(1),  // object type index
        SET_OBJ_FIELD(2),  // offset from pointer, size
        GET_OBJ_FIELD(2), // offset from pointer, size

        ALLOCATE_LIST(2),  // list_item_type, initial_list_size
        // no operands, because list pointer is on the stack,
        // and list index value is calculated on stack as well
        GET_LIST_ITEM(0),
        SET_LIST_ITEM(2),  // offset from pointer, size of value to set

        // ACTIVE-RELATED OPCODES
        SPAWN(3),  // type index, call position (u16)
        // CURRENT_ACTIVE(0),
        // GET_CURRENT_ACTIVE_FIELD(2), // offset from pointer, size
        // SET_CURRENT_ACTIVE_FIELD(2), // offset from pointer, size
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
