const STACK_SIZE: usize = 256;

static start: usize = 0;

macro_rules! next_opcode {
    ($start:ident) => {
        $start += 1;
        $start
    };
}

pub mod Op {
    pub type Opcode = u8;

    pub const LOAD_CONST: u8 = 0;  // 1 arg - const index
    pub const LOAD_INT: u8 = 1;  // 1 arg - small int
    
    pub const ADD_INT: u8 = 2;  // 0 args
    pub const SUB_INT: u8 = 3;  // 0 args
    pub const MUL_INT: u8 = 4;  // 0 args
    pub const DIV_INT: u8 = 5;  // 0 args

    pub const ADD_FLOAT: u8 = 6;  // 0 args
    pub const SUB_FLOAT: u8 = 7;  // 0 args
    pub const MUL_FLOAT: u8 = 8;  // 0 args
    pub const DIV_FLOAT: u8 = 9;  // 0 args

    pub const CALL: u8 = 10;  // 1 arg - args amount
    pub const RETURN: u8 = 11;  // 0 args
    pub const POP: u8 = 12;  // 0 args

    pub const SET_VAR: u8 = 13;  // 1 arg - offset which to load
    pub const GET_VAR: u8 = 14;  // 1 arg - offset where to save
}

// TODO: nan boxing?
type Value = u64;

struct CallFrame {
    return_ip: usize,
}

struct Vm {
    program: Vec<u8>,
    stack: [Value; STACK_SIZE],
    stack_pointer: usize,
    frames: Vec<CallFrame>, // TODO: limit size
}

impl Vm {
    pub fn new(program: Vec<u8>) -> Self {
        Vm { program, stack: [0; STACK_SIZE], stack_pointer: 0, frames: vec![] }
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_pointer] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer]
    }

    pub fn read_opcode(&mut self) -> u8 {
        let byte = self.program[self.stack_pointer];
        self.stack_pointer += 1;
        byte
    }

    pub fn run(&mut self) {
        let mut pc = 0;
        while pc < self.program.len() {
            let opcode = self.read_opcode();
            match opcode {
                Op::LOAD_INT => {
                    let value = self.program[pc + 1];
                    self.stack[self.stack_pointer] = value as u64;
                    self.stack_pointer += 1;
                    pc += 2;
                }
                Op::ADD_INT => {
                    let a = self.pop() as i64;
                    let b = self.pop() as i64;
                    self.push((a + b) as Value);
                }
                Op::ADD_FLOAT => {
                    let a = self.pop() as i64;
                    let b = self.pop() as i64;
                    self.push((a + b) as Value);
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}
