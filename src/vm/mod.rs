const STACK_SIZE: usize = 256;

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

enum Opcode {
    Load = 1,
    Load_Int, // load int from -128 to 127
    Add_i,
    Add_f,
    Call,
    Return,
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

    pub fn read_next(&mut self) -> u8 {
        let byte = self.program[self.stack_pointer];
        self.stack_pointer += 1;
        byte
    }

    pub fn run(&mut self) {
        let mut pc = 0;
        while pc < self.program.len() {
            match self.read_next() {
                Load_int => {
                    let value = self.program[pc + 1];
                    self.stack[self.stack_pointer] = value;
                    self.stack_pointer += 1;
                    pc += 2;
                }
                Add_i | Add_f => {
                    let a = self.pop();
                    let b = self.pop();
                    let result: Value = match opcode {
                        Add_i => (a as i64) + (b as i64),
                        Add_f => (a as f64) + (b as f64),
                    } as Value;
                    self.push(result);
                    pc += 1;
                }
            }
        }
    }
}
