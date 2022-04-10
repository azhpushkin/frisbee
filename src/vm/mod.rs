const STACK_SIZE: usize = 256;

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
    opcodes_list!(
        0,
        LOAD_CONST,
        LOAD_INT,

        ADD_INT,
        SUB_INT,
        MUL_INT,
        DIV_INT,

        ADD_FLOAT,
        SUB_FLOAT,
        MUL_FLOAT,
        DIV_FLOAT,

        CALL,
        RETURN,
        POP,
        SET_VAR,
        GET_VAR
    );

    pub const CONST_END_FLAG: u8 = 0;
    pub const CONST_INT_FLAG: u8 = 1;    
    pub const CONST_FLOAT_FLAG: u8 = 2;
    pub const CONST_STRING_FLAG: u8 = 3;
}

struct CallFrame {
    return_ip: usize,
}

pub struct Vm {
    program: Vec<u8>,
    stack: [u64; STACK_SIZE],
    stack_pointer: usize,
    frames: Vec<CallFrame>, // TODO: limit size
}

impl Vm {
    pub fn new(program: Vec<u8>) -> Self {
        Vm { program, stack: [0; STACK_SIZE], stack_pointer: 0, frames: vec![] }
    }

    fn push(&mut self, value: u64) {
        self.stack[self.stack_pointer] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u64 {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer]
    }

    fn read_opcode(&mut self) -> u8 {
        let byte = self.program[self.stack_pointer];
        self.stack_pointer += 1;
        byte
    }

    pub fn run(&mut self) {
        let mut pc = 0;
        while pc < self.program.len() {
            let opcode = self.read_opcode();
            match opcode {
                op::LOAD_INT => {
                    let value = self.program[pc + 1];
                    self.stack[self.stack_pointer] = value as u64;
                    self.stack_pointer += 1;
                    pc += 2;
                }
                op::ADD_INT => {
                    let a = self.pop() as i64;
                    let b = self.pop() as i64;
                    self.push((a + b) as u64);
                }
                op::ADD_FLOAT => {
                    let a = self.pop() as i64;
                    let b = self.pop() as i64;
                    self.push((a + b) as u64);
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}