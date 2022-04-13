const ONE_ARGS_OPCODES: u8 = 100;
const TWO_ARGS_OPCODES: u8 = 200;

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

        ADD_INT,
        SUB_INT,
        MUL_INT,
        DIV_INT,

        ADD_FLOAT,
        SUB_FLOAT,
        MUL_FLOAT,
        DIV_FLOAT,

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
    opcodes_list!(200,
        CALL
    );

    pub const CONST_END_FLAG: u8 = 0;
    pub const CONST_INT_FLAG: u8 = 1;    
    pub const CONST_FLOAT_FLAG: u8 = 2;
    pub const CONST_STRING_FLAG: u8 = 3;
}

pub fn args_num(op: u8) -> usize {
    if op < ONE_ARGS_OPCODES {
        return 0;
    } else if op < TWO_ARGS_OPCODES {
        return 1;
    } else {
        return 2;
    }
}