const ZERO_ARG_START: u8 = 0;
const SINGLE_ARG_START: u8 = 128;
const DOUBLE_ARG_START: u8 = 192;
const TRIPLE_ARG_START: u8 = 224;
const QUATRO_ARG_START: u8 = 240;

macro_rules! opcodes_list {
    ($s:expr, $c:ident ) => {
        pub const $c: u8 = $s;
    };
    ($s:expr, $c:ident, $ ( $more:ident ),+ ) => {
        pub const $c: u8 = $s;

        opcodes_list!($s+1, $(  $more ),+ );
    };
}

macro_rules! opcodes_with_0_args {
    ($ ( $more:ident ),+ ) => { opcodes_list!(ZERO_ARG_START, $(  $more ),+ ); };
}
macro_rules! opcodes_with_1_arg {
    ($ ( $more:ident ),+ ) => { opcodes_list!(SINGLE_ARG_START, $(  $more ),+ ); };
}
macro_rules! opcodes_with_2_args {
    ($ ( $more:ident ),+ ) => { opcodes_list!(DOUBLE_ARG_START, $(  $more ),+ ); };
}
macro_rules! opcodes_with_3_args {
    ($ ( $more:ident ),+ ) => { opcodes_list!(TRIPLE_ARG_START, $(  $more ),+ ); };
}
macro_rules! opcodes_with_4_args {
    ($ ( $more:ident ),+ ) => { opcodes_list!(QUATRO_ARG_START, $(  $more ),+ ); };
}

#[rustfmt::skip]
pub mod op {
    use super::*;

    // Opcode without arguments
    opcodes_with_0_args!(
        LOAD_TRUE,
        LOAD_FALSE,

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

        ADD_STRINGS,
        EQ_STRINGS,

        GET_LIST_ITEM,

        RETURN
    );

    // Opcodes with single operand
    opcodes_with_1_arg!(
        ALLOCATE,

        RESERVE,
        POP,

        LOAD_CONST,
        LOAD_SMALL_INT
    );

    // Opcodes with two operands
    opcodes_with_2_args!(
        ALLOCATE_LIST,  // item_size + initial_size

        JUMP,
        JUMP_BACK,
        JUMP_IF_FALSE,

        SET_LOCAL,  // offset + size
        GET_LOCAL,  // offset + size

        SET_OBJ_FIELD,  // offset from pointer, size
        GET_OBJ_FIELD, // offset from pointer, size

        SET_LIST_ITEM  // offset from pointer, size of value to set
    );

    // Opcodes with three operands
    opcodes_with_3_args!(
        GET_TUPLE_ITEM
    );

    // Both have 4 operands: return size, total offset, call position
    opcodes_with_4_args! (
        CALL,
        CALL_STD
    );

    pub const CONST_END_FLAG: u8 = u8::MAX;
    pub const CONST_INT_FLAG: u8 = 1;    
    pub const CONST_FLOAT_FLAG: u8 = 2;
    pub const CONST_STRING_FLAG: u8 = 3;
}

pub fn get_args_num(op: u8) -> usize {
    if op >= QUATRO_ARG_START {
        4
    } else if op >= TRIPLE_ARG_START {
        3
    } else if op >= DOUBLE_ARG_START {
        2
    } else if op >= SINGLE_ARG_START {
        1
    } else {
        0
    }
}
