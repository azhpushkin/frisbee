use super::opcodes::op;

const STACK_SIZE: usize = 1024;

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
        self.frames.push(CallFrame {
            return_ip: self.ip + 1,
            args_num,
            stack_start: self.stack_pointer,
        });
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

    fn read_several<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0; N];
        for i in 0..N {
            bytes[i] = self.read_opcode();
        }
        bytes
    }

    fn read_two_values<T>(&mut self) -> (T, T)
    where
        T: From<u8>,
    {
        let values = self.read_several::<2>();
        (T::from(values[0]), T::from(values[1]))
    }

    fn exec_unaryop<T>(&mut self, op: fn(T) -> u64)
    where
        T: From<u8>,
    {
        let value = T::from(self.read_opcode());
        self.push(op(value));
    }

    fn exec_binaryop<T>(&mut self, op: fn(T, T) -> u64)
    where
        T: From<u8>,
    {
        let (a, b) = self.read_two_values::<T>();
        self.push(op(a, b));
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
                op::NEGATE_INT => self.exec_unaryop::<i64>(|x| (-x) as u64),
                op::ADD_INT => self.exec_binaryop::<i64>(|a, b| (a + b) as u64),
                op::MUL_INT => self.exec_binaryop::<i64>(|a, b| (a * b) as u64),
                op::SUB_INT => self.exec_binaryop::<i64>(|a, b| (a - b) as u64),
                op::DIV_INT => self.exec_binaryop::<i64>(|a, b| (a / b) as u64),
                op::GREATER_INT => self.exec_binaryop::<i64>(|a, b| (a > b) as u64),
                op::LESS_INT => self.exec_binaryop::<i64>(|a, b| (a < b) as u64),
                op::EQ_INT => self.exec_binaryop::<i64>(|a, b| (a == b) as u64),

                op::NEGATE_FLOAT => self.exec_unaryop::<f64>(|x| (-x) as u64),
                op::ADD_FLOAT => self.exec_binaryop::<f64>(|a, b| (a + b) as u64),
                op::MUL_FLOAT => self.exec_binaryop::<f64>(|a, b| (a * b) as u64),
                op::SUB_FLOAT => self.exec_binaryop::<f64>(|a, b| (a - b) as u64),
                op::DIV_FLOAT => self.exec_binaryop::<f64>(|a, b| (a / b) as u64),
                op::GREATER_FLOAT => self.exec_binaryop::<f64>(|a, b| (a > b) as u64),
                op::LESS_FLOAT => self.exec_binaryop::<f64>(|a, b| (a < b) as u64),
                op::EQ_FLOAT => self.exec_binaryop::<f64>(|a, b| (a == b) as u64),

                // TODO: test bool operators
                op::NEGATE_BOOL => self.exec_unaryop::<u64>(|x| !x),
                op::AND_BOOL => self.exec_binaryop::<u64>(|a, b| a & b),
                op::OR_BOOL => self.exec_binaryop::<u64>(|a, b| a | b),
                op::EQ_BOOL => self.exec_binaryop::<u64>(|a, b| !(a ^ b)),

                op::GET_VAR => {
                    let value_pos = self.read_opcode() as usize;
                    self.push(
                        self.stack[self.current_frame().stack_start + value_pos
                            - self.current_frame().args_num],
                    );
                }
                op::SET_VAR => {
                    let value = self.pop();
                    let value_pos = self.read_opcode() as usize;
                    self.stack[self.current_frame().stack_start + value_pos
                        - self.current_frame().args_num] = value;
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
