pub mod opcodes;

use opcodes::op;

const STACK_SIZE: usize = 256;


struct CallFrame {
    return_ip: usize,
}

pub struct Vm {
    program: Vec<u8>,
    constants: Vec<u64>,
    stack: [u64; STACK_SIZE],
    stack_pointer: usize,
    frames: Vec<CallFrame>, // TODO: limit size
}

impl Vm {
    pub fn new(program: Vec<u8>) -> Self {
        Vm { program, constants: vec![], stack: [0; STACK_SIZE], stack_pointer: 0, frames: vec![] }
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
                op::LOAD_CONST => {
                    let index = self.read_opcode();
                    self.push(self.constants[index as usize]);
                }
                op::LOAD_INT => {
                    let value = self.read_opcode();
                    self.push(value as u64);
                }
                op::ADD_INT => {
                    let a = self.pop() as i64;
                    let b = self.pop() as i64;
                    self.push((a + b) as u64);
                }
                op::MUL_INT => {
                    let a = self.pop() as i64;
                    let b = self.pop() as i64;
                    self.push((a * b) as u64);
                }
                op::ADD_FLOAT => {
                    let a = self.pop() as f64;
                    let b = self.pop() as f64;
                    self.push((a + b) as u64);
                }
                op::GET_VAR => {
                    let value_pos = self.read_opcode() as usize;
                    self.push(self.stack[value_pos]);
                }
                op::SET_VAR => {
                    let value = self.pop();
                    let value_pos = self.read_opcode() as usize;
                    self.stack[value_pos] = value;
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}
