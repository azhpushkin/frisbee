pub mod opcodes;

use opcodes::op;

const STACK_SIZE: usize = 256;

struct CallFrame {
    pub return_ip: usize,
    pub stack_start: usize,
    pub args_num: usize,
}

pub struct Vm {
    program: Vec<u8>,
    ip: usize,
    constants: Vec<u64>,
    stack: [u64; STACK_SIZE],
    stack_pointer: usize,
    frames: Vec<CallFrame>, // TODO: limit size
}

impl Vm {
    pub fn new(program: Vec<u8>) -> Self {
        Vm {
            program,
            ip: 0,
            constants: vec![],
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            frames: vec![],
        }
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
        let byte = self.program[self.ip];
        self.ip += 1;
        byte
    }

    fn call_op(&mut self, func_pos: usize, args_num: usize) {
        self.frames.push(CallFrame { return_ip: self.ip+1, args_num, stack_start: self.stack_pointer });
        self.ip = func_pos;
    }

    fn return_op(&mut self) {
        let frame = self.frames.pop().unwrap();
        self.ip = frame.return_ip;
        self.stack_pointer = frame.stack_start;
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    pub fn run(&mut self) {
        
        let mut pc = 0;
        while pc < self.program.len() {
            println!(">> pc: {:02x?}", pc);
            println!("  stack: {:02x?}", &self.stack[0..self.stack_pointer]);
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
                    self.push(self.stack[value_pos - self.current_frame().args_num]);
                }
                op::SET_VAR => {
                    let value = self.pop();
                    let value_pos = self.read_opcode() as usize;
                    self.stack[value_pos - self.current_frame().args_num] = value;
                }
                op::RESERVE_ONE => {
                    // TODO: this seems wrong, function might reserve at the very start tbh
                    // should check after basic implementation probably
                    self.push(0);
                }
                op::CALL => {
                    let args_num = self.read_opcode();
                    let function_pos = u16::from_be_bytes([self.read_opcode(), self.read_opcode()]);
                    self.call_op(function_pos as usize, args_num as usize);
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}
