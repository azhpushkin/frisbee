/*

opcodes(
    LOAD_VALUE(1),
    LOAD_ANOTHER(2),
)


const LOAD_VALUE: u8 = 1;



*/

macro_rules! opcodes_list {
    ($s:expr, $c:ident($args:literal)  ) => {
        pub const $c: u8 = $s;
    };
    ($s:expr, $c:ident($args:literal), $ ( $more:ident($margs:literal) ),+ $(,)? ) => {
        pub const $c: u8 = $s;

        opcodes_list!($s+1, $(  $more($margs) ),+ );
    };
}


mod op {
    macro_rules! args_num {
        (op::$s:ident) => {
            $s::ARGS
        };
    }

    pub(crate) use args_num;


    opcodes_list!(
        0,
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
}

