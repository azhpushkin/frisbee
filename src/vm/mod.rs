const STACK_SIZE: usize = 256;

pub mod Op {
    pub const LOAD: u8 = 0;
    pub const LOAD_INT: u8 = 1;
    pub const ADD_INT: u8 = 2;
    pub const ADD_FLOAT: u8 = 3;
    pub const CALL: u8 = 4;
    pub const RETURN: u8 = 5;
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
