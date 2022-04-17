macro_rules! opcodes_list {
    ($s:expr, $c:ident ) => {
        pub const $c: u8 = $s;
    };
    ($s:expr, $c:ident, $ ( $more:ident ),+ ) => {
        pub const $c: u8 = $s;

        opcodes_list!($s+1, $(  $more ),+ );
    };
}

#[rustfmt::skip]
pub mod op {
    // Opcode without arguments
    opcodes_list!(0,
        LOAD_TRUE,
        LOAD_FALSE,

        RESERVE_ONE,

        NEGATE_INT,
        ADD_INT,
        SUB_INT,
        MUL_INT,
        DIV_INT,
        GREATER_INT,
        LESS_INT,
        EQ_INT,

        NEGATE_FLOAT,
        ADD_FLOAT,
        SUB_FLOAT,
        MUL_FLOAT,
        DIV_FLOAT,
        GREATER_FLOAT,
        LESS_FLOAT,
        EQ_FLOAT,

        NEGATE_BOOL,
        EQ_BOOL,
        AND_BOOL,
        OR_BOOL,

        RETURN,
        POP
    );

    // Opcodes with single operand
    opcodes_list!(100,
        LOAD_CONST,
        LOAD_INT,
        SET_VAR,
        GET_VAR
    );

    // Opcodes with two operands
    opcodes_list!(180,
        JUMP,
        JUMP_BACK,
        JUMP_IF_FALSE
    );

    opcodes_list!(220,
        CALL
    );

    pub const CONST_END_FLAG: u8 = 0;
    pub const CONST_INT_FLAG: u8 = 1;    
    pub const CONST_FLOAT_FLAG: u8 = 2;
    pub const CONST_STRING_FLAG: u8 = 3;
}

pub fn get_args_num(op: u8) -> usize {
    if op < 100 {
        return 0;
    } else if op < 180 {
        return 1;
    } else if op < 220 {
        return 2;
    } else {
        return 3;
    }
}
