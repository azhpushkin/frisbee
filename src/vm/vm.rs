use super::opcodes::op;

const STACK_SIZE: usize = 1024;

struct CallFrame {
    pub return_ip: usize,
    pub stack_start: usize,
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
        self.frames
            .push(CallFrame { return_ip: self.ip, stack_start: self.stack_pointer - args_num - 1 });
        self.ip = func_pos;
    }

    fn return_op(&mut self) {
        let frame = self.frames.pop().unwrap();
        self.ip = frame.return_ip;
        self.stack_pointer = frame.stack_start + 1;
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn read_several<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes: [u8; N] = [0; N];
        for i in 0..N {
            bytes[i] = self.read_opcode();
        }
        bytes
    }

    fn exec_binaryop_i64(&mut self, op: fn(i64, i64) -> i64) {
        let a = self.pop() as i64;
        let b = self.pop() as i64;
        self.push(op(a, b) as u64);
    }

    fn exec_binaryop_f64(&mut self, op: fn(f64, f64) -> f64) {
        let a = self.pop() as f64;
        let b = self.pop() as f64;
        self.push(op(a, b) as u64);
    }

    fn exec_binaryop(&mut self, op: fn(u64, u64) -> u64) {
        let a = self.pop();
        let b = self.pop();
        self.push(op(a, b));
    }

    fn exec_unaryop(&mut self, op: fn(u64) -> u64) {
        let a = self.pop();
        self.push(op(a));
    }

    fn load_consts(&mut self) {
        loop {
            let const_type = self.read_opcode();
            match const_type {
                op::CONST_INT_FLAG => {
                    let i = i64::from_be_bytes(self.read_several::<8>());
                    self.constants.push(i as u64);
                }
                op::CONST_FLOAT_FLAG => {
                    let f = f64::from_be_bytes(self.read_several::<8>());
                    self.constants.push(f as u64);
                }
                op::CONST_STRING_FLAG => {
                    let str_len = u16::from_be_bytes(self.read_several::<2>());
                    panic!("Strings are not implemented yet!");
                }
                op::CONST_END_FLAG => break,
                c => panic!("Unknown const flag: {:02x}", c),
            };
        }
        println!("Loaded constants: {:?}", self.constants);
    }

    pub fn run(&mut self) {
        self.load_consts();

        let entry = u16::from_be_bytes(self.read_several::<2>());
        self.push(0); // return address
        self.call_op(entry as usize, 0);

        while self.ip < self.program.len() {
            println!("  stack: {:02x?}", &self.stack[0..self.stack_pointer]);
            println!(">> exec pc: {:02x?}", self.ip);

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

                // TODO: test div and suband compare for float and ints
                op::NEGATE_INT => self.exec_unaryop(|x| (-(x as i64)) as u64),
                op::ADD_INT => self.exec_binaryop_i64(|a, b| a + b),
                op::MUL_INT => self.exec_binaryop_i64(|a, b| a * b),
                op::SUB_INT => self.exec_binaryop_i64(|a, b| a - b),
                op::DIV_INT => self.exec_binaryop_i64(|a, b| a / b),
                op::GREATER_INT => self.exec_binaryop(|a, b| ((a as i64) > (b as i64)) as u64),
                op::LESS_INT => self.exec_binaryop(|a, b| ((a as i64) < (b as i64)) as u64),
                op::EQ_INT => self.exec_binaryop(|a, b| ((a as i64) == (b as i64)) as u64),

                op::NEGATE_FLOAT => self.exec_unaryop(|x| (-(x as f64)) as u64),
                op::ADD_FLOAT => self.exec_binaryop_f64(|a, b| a + b),
                op::MUL_FLOAT => self.exec_binaryop_f64(|a, b| a * b),
                op::SUB_FLOAT => self.exec_binaryop_f64(|a, b| a - b),
                op::DIV_FLOAT => self.exec_binaryop_f64(|a, b| a / b),
                op::GREATER_FLOAT => self.exec_binaryop(|a, b| ((a as f64) > (b as f64)) as u64),
                op::LESS_FLOAT => self.exec_binaryop(|a, b| ((a as f64) < (b as f64)) as u64),
                op::EQ_FLOAT => self.exec_binaryop(|a, b| ((a as f64) == (b as f64)) as u64),

                // TODO: test bool operators
                op::NEGATE_BOOL => self.exec_unaryop(|x| !x),
                op::AND_BOOL => self.exec_binaryop(|a, b| a & b),
                op::OR_BOOL => self.exec_binaryop(|a, b| a | b),
                op::EQ_BOOL => self.exec_binaryop(|a, b| !(a ^ b)),
                op::GET_VAR => {
                    let value_pos = self.read_opcode() as usize;
                    self.push(self.stack[self.current_frame().stack_start + value_pos]);
                }
                op::SET_VAR => {
                    let value = self.pop();
                    let value_pos = self.read_opcode() as usize;
                    self.stack[self.current_frame().stack_start + value_pos] = value;
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
                op::POP => {
                    self.pop();
                }
                op::RETURN => {
                    self.return_op();
                    if self.frames.len() == 0 {
                        println!(
                            "VM finished, stack is {:?}",
                            &self.stack[0..self.stack_pointer]
                        );
                        break;
                    }
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}
