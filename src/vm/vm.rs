use super::opcodes::op;
use super::stdlib_runners::STD_RAW_FUNCTION_RUNNERS;
use super::utils::{u64_to_f64, f64_to_u64};

const STACK_SIZE: usize = 1024;

struct CallFrame {
    pub return_ip: usize,
    pub stack_start: usize,
}


pub struct Vm {
    program: Vec<u8>,
    ip: usize,
    constants: Vec<u64>,
    strings: Vec<String>, //  DO NOT EMPTY IT YET
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
            strings: vec![],
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

    fn call_std(&mut self, func_index: usize, args_num: usize) {
        let start = self.stack_pointer - args_num - 1;
        STD_RAW_FUNCTION_RUNNERS[func_index].1(&mut self.stack[start..], &mut self.strings);
        self.stack_pointer -= args_num;
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

    fn read_bytes(&mut self, num: usize) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        for _ in 0..num {
            bytes.push(self.read_opcode());
        }
        bytes
    }

    // NOTE: items are pushed on stack in-order (from left to right)
    // which means that they are popped in reverse order (from right to left)
    // so first pop b, than pop a
    fn exec_binaryop_i64(&mut self, op: fn(i64, i64) -> i64) {
        let b = self.pop() as i64;
        let a = self.pop() as i64;
        self.push(op(a, b) as u64);
    }

    fn exec_binaryop_f64(&mut self, op: fn(f64, f64) -> f64) {
        let b = u64_to_f64(self.pop());
        let a = u64_to_f64(self.pop());
        let res = op(a, b);
        self.push(f64_to_u64(res));
    }

    fn exec_binaryop(&mut self, op: fn(u64, u64) -> u64) {
        let b = self.pop();
        let a = self.pop();
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
                    let f = u64::from_be_bytes(self.read_several::<8>());
                    self.constants.push(f);
                }
                op::CONST_STRING_FLAG => {
                    let str_len = u16::from_be_bytes(self.read_several::<2>());
                    let str_bytes = self.read_bytes(str_len as usize);
                    self.strings.push(String::from_utf8(str_bytes).unwrap());
                    self.constants.push((self.strings.len() - 1) as u64);
                }
                op::CONST_END_FLAG => break,
                c => panic!("Unknown const flag: {:02x}", c),
            };
        }
        self.check_header("End of constants table");
        println!("Loaded constants: {:?}", self.constants);
    }

    fn check_header(&mut self, header_name: &'static str) {
        let header = self.read_several::<2>();
        if header != [0xff, 0xff] {
            panic!("Cannot find header: {}", header_name);
        }
    }

    fn skip_symbol_names(&mut self) {
        loop {
            let symbol_name_len = u16::from_be_bytes(self.read_several::<2>());
            if symbol_name_len == 0 {
                break;
            }
            self.read_bytes(symbol_name_len as usize); // symbol name
            self.read_several::<2>(); // symbol position
        }
        self.check_header("End of symbol names");
    }

    fn load_entry(&mut self) -> usize {
        let entry = u16::from_be_bytes(self.read_several::<2>());
        self.check_header("Entry loaded, start of functions");
        entry as usize
    }

    pub fn run(&mut self) {
        self.check_header("Initial header");
        self.load_consts();
        self.skip_symbol_names();
        let entry = self.load_entry();

        self.push(0); // return address for entry function
        self.call_op(entry, 0);

        while self.ip < self.program.len() {
            println!("  stack: {:02x?}", &self.stack[0..self.stack_pointer]);
            println!(">> exec pc: {:02x?}", self.ip);

            let opcode = self.read_opcode();
            match opcode {
                op::LOAD_CONST => {
                    let index = self.read_opcode();
                    self.push(self.constants[index as usize]);
                }
                op::LOAD_SMALL_INT => {
                    let value = self.read_opcode();
                    self.push(value as u64);
                }
                op::LOAD_TRUE => {
                    self.push(1);
                }
                op::LOAD_FALSE => {
                    self.push(0);
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

                op::NEGATE_FLOAT => self.exec_unaryop(|x| f64_to_u64(-u64_to_f64(x))),
                op::ADD_FLOAT => self.exec_binaryop_f64(|a, b| a + b),
                op::MUL_FLOAT => self.exec_binaryop_f64(|a, b| a * b),
                op::SUB_FLOAT => self.exec_binaryop_f64(|a, b| a - b),
                op::DIV_FLOAT => self.exec_binaryop_f64(|a, b| a / b),
                op::GREATER_FLOAT => self.exec_binaryop(|a, b| (u64_to_f64(a) > u64_to_f64(b)) as u64),
                op::LESS_FLOAT => self.exec_binaryop(|a, b| ((a as f64) < (b as f64)) as u64),
                op::EQ_FLOAT => self.exec_binaryop(|a, b| ((a as f64) == (b as f64)) as u64),

                // TODO: test bool operators
                op::NEGATE_BOOL => self.exec_unaryop(|x| !x),
                op::AND_BOOL => self.exec_binaryop(|a, b| a & b),
                op::OR_BOOL => self.exec_binaryop(|a, b| a | b),
                op::EQ_BOOL => self.exec_binaryop(|a, b| !(a ^ b)),

                op::ADD_STRINGS => {
                    let (b, a) = (self.pop(), self.pop());
                    let (s1, s2) = (&self.strings[a as usize], &self.strings[b as usize]);
                    let res = format!("{}{}", s1, s2);
                    self.strings.push(res);
                    self.push((self.strings.len() - 1) as u64);
                }
                op::EQ_STRINGS => {
                    let (b, a) = (self.pop(), self.pop());
                    let (s1, s2) = (&self.strings[a as usize], &self.strings[b as usize]);
                    let res = (s1 == s2) as u64;
                    self.push(res);
                }

                op::GET_VAR => {
                    let value_pos = self.read_opcode() as usize;
                    let value_size = self.read_opcode() as usize;
                    for i in 0..value_size {
                        self.push(self.stack[self.current_frame().stack_start + value_pos + i]);
                    }
                }
                op::SET_VAR => {
                    let value_pos = self.read_opcode() as usize;
                    let value_size = self.read_opcode() as usize;
                    // Go backwards because pop() returns items in a reversed order
                    for i in 0..value_size {
                        let value = self.pop();
                        self.stack[self.current_frame().stack_start + value_pos + value_size-i-1] = value;
                    }
                }
                op::RESERVE => {
                    // TODO: this seems wrong, function might reserve at the very start tbh
                    // should check after basic implementation probably
                    let value = self.read_opcode();
                    for _ in 0..value {
                        self.push(0);
                    } 
                }
                op::CALL => {
                    let args_num = self.read_opcode();
                    let function_pos = u16::from_be_bytes([self.read_opcode(), self.read_opcode()]);
                    self.call_op(function_pos as usize, args_num as usize);
                }
                op::CALL_STD => {
                    let args_num = self.read_opcode();
                    let std_function_index = self.read_opcode();
                    self.call_std(std_function_index as usize, args_num as usize);
                }
                op::POP => {
                    let amount = self.read_opcode();
                    for _ in 0..amount {
                        self.pop();
                    }
                    
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
                op::JUMP => {
                    let x = u16::from_be_bytes(self.read_several::<2>());
                    self.ip += x as usize;
                }
                op::JUMP_IF_FALSE => {
                    let c = self.pop();
                    let x = u16::from_be_bytes(self.read_several::<2>());
                    if c == 0 {
                        self.ip += x as usize;
                    }
                }
                op::JUMP_BACK => {
                    let x = u16::from_be_bytes(self.read_several::<2>());
                    self.ip -= x as usize;
                }
                _ => panic!("Unknown opcode: {}", opcode),
            }
        }
    }
}
