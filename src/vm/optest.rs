/*

opcodes(
    LOAD_VALUE(1),
    LOAD_ANOTHER(2),
)


const LOAD_VALUE: u8 = 1;



*/

macro_rules! opcodes_list {
    ($s:expr, [$($acc:expr),*], $c:ident($args:literal)  ) => {
        pub const $c: u8 = $s;

        const ARGS_NUM: [u8; $s+1] = [$($acc),*, $args];
    };
    ($s:expr, [$($acc:expr),+], $c:ident($args:literal), $ ( $more:ident($margs:literal) ),+ $(,)? ) => {
        pub const $c: usize = $s;

        opcodes_list!($s+1, [$($acc),*, $args], $(  $more($margs) ),+ );
    };

    // Empty array call
    ($s:expr, [], $c:ident($args:literal), $ ( $more:ident($margs:literal) ),+ $(,)? ) => {
        pub const $c: u8 = $s;

        opcodes_list!($s+1, [$args], $(  $more($margs) ),+ );
    };
    
    // Initial call
    ($c:ident($args:literal), $ ( $more:ident($margs:literal) ),+ $(,)? ) => {
        opcodes_list!(0, [], $c($args), $(  $more($margs) ),+ );
    };
}


mod op {
    // macro_rules! args_num {
    //     (op::$s:ident) => {
    //         $s::ARGS
    //     };
    // }

    // pub(crate) use args_num;
    pub fn get_args_num(opcode: u8) -> usize {
        ARGS_NUM[opcode as usize] as usize
    }

    opcodes_list!(
        LOAD_VALUE(3),
        LOAD_ANOTHER(12),
        X(2),
        Y(0),
    );
}

mod test {
    use super::*;

    #[test]
    fn test_opcodes() {
        assert_eq!(op::LOAD_VALUE, 0);
        assert_eq!(op::LOAD_ANOTHER, 1);
        assert_eq!(op::X, 2);
        assert_eq!(op::Y, 3);
    }

    #[test]
    fn test_args_num() {
        assert_eq!(op::get_args_num[op::LOAD_VALUE as usize], 3);
        assert_eq!(op::get_args_num[op::LOAD_ANOTHER as usize], 12);
        assert_eq!(op::get_args_num[op::X as usize], 2);
        assert_eq!(op::get_args_num[op::Y as usize], 0);
    }
}

