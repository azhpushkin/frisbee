const STACK_SIZE = 256;


type Value = u64;

struct Vm {
    program: Vec<u8>,
    stack: [Value; STACK_SIZE],
    stack_pointer: usize,
}




enum Opcode {
    Load,
    Add,
    Return
}

impl Vm {
    pub fn new(program: Vec<u8>) -> Self {
        Vm {
            program,
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
        }
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_pointer] = value;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer]
    }

    pub fn run(&mut self) {
        let mut pc = 0;
        while pc < self.program.len() {
            let opcode = self.program[pc];
            match opcode {
                Load => {
                    let value = self.program[pc + 1];
                    self.stack[self.stack_pointer] = value;
                    self.stack_pointer += 1;
                    pc += 2;
                }
            }
        }
    }
}