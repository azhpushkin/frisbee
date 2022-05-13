use std::io;

use super::heap;
use super::metadata::{Metadata, MetadataBlock};
use super::opcodes::op;
use super::stdlib_runners::STD_RAW_FUNCTION_RUNNERS;
use super::utils::{f64_to_u64, u64_to_f64};

macro_rules! push {
    ($vm:ident, $value:expr) => {
        $vm.stack[$vm.stack_pointer] = $value;
        $vm.stack_pointer += 1;
    };
}

const STACK_SIZE: usize = 512; // TODO: maybe grow??

struct CallFrame {
    pub pos: usize,
    pub return_ip: usize,
    pub stack_start: usize,
    pub return_size: usize,
}

pub struct Vm {
    program: Vec<u8>,
    ip: usize,

    constants: Vec<u64>,
    memory: heap::Heap,
    metadata: &'static mut Metadata,

    stack: [u64; STACK_SIZE],
    stack_pointer: usize,

    frames: Vec<CallFrame>, // TODO: limit size
}

impl Vm {
    pub fn new(program: Vec<u8>) -> Self {
        let metadata = Box::new(Metadata::default());
        Vm {
            program,
            ip: 0,
            constants: vec![],
            memory: heap::Heap::new(),
            metadata: Box::leak(metadata),
            stack: [0; STACK_SIZE],
            stack_pointer: 0,
            frames: vec![],
        }
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

    fn call_op(&mut self, func_pos: usize, return_size: usize, locals_size: usize) {
        // This is a point of huge optimizations, probably worth tweaking callframe stack
        // to make it smaller
        self.frames.push(CallFrame {
            pos: func_pos,
            return_ip: self.ip,
            stack_start: self.stack_pointer - locals_size - return_size,
            return_size,
        });
        self.ip = func_pos;
    }

    fn call_std(&mut self, func_index: usize, return_size: usize, locals_size: usize) {
        let start = self.stack_pointer - locals_size - return_size;
        STD_RAW_FUNCTION_RUNNERS[func_index].1(
            &mut self.stack[start..self.stack_pointer],
            &mut self.memory,
            self.metadata
        );
        self.stack_pointer = start + return_size;
    }

    fn return_op(&mut self) {
        let frame = self.frames.pop().unwrap();
        self.ip = frame.return_ip;
        self.stack_pointer = frame.stack_start + frame.return_size;
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn read_several<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes: [u8; N] = [0; N];
        for byte in bytes.iter_mut() {
            *byte = self.read_opcode();
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
        push!(self, op(a, b) as u64);
    }

    fn exec_binaryop_f64(&mut self, op: fn(f64, f64) -> f64) {
        let b = u64_to_f64(self.pop());
        let a = u64_to_f64(self.pop());
        let res = op(a, b);
        push!(self, f64_to_u64(res));
    }

    fn exec_binaryop(&mut self, op: fn(u64, u64) -> u64) {
        let b = self.pop();
        let a = self.pop();
        push!(self, op(a, b));
    }

    fn exec_unaryop(&mut self, op: fn(u64) -> u64) {
        let a = self.pop();
        push!(self, op(a));
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

                    let (obj_pos, inner) = self.memory.allocate_string(str_len as usize);
                    inner.extend(std::str::from_utf8(&str_bytes).unwrap().chars());
                    self.constants.push(obj_pos);
                }
                op::CONST_END_FLAG => break,
                c => panic!("Unknown const flag: {:02x}", c),
            };
        }
        self.check_header("End of constants table");
    }

    fn check_header(&mut self, header_name: &'static str) {
        let header = self.read_several::<2>();
        if header != [0xff, 0xff] {
            panic!("Cannot find header: {}", header_name);
        }
    }

    fn read_metadata_block(&mut self, info_name: &'static str) -> MetadataBlock {
        let amount = self.read_opcode();
        let mut res = vec![];
        for _ in 0..amount {
            // Read string repr, do not save it - we have no use for it
            let symbol_name_len = u16::from_be_bytes(self.read_several::<2>());
            self.read_bytes(symbol_name_len as usize);

            let flag = u16::from_be_bytes(self.read_several::<2>()) as usize;

            let pointers_amount = self.read_opcode();
            let pointer_mapping = self.read_bytes(pointers_amount as usize);
            res.push((flag, pointer_mapping));
        }
        self.check_header(info_name);
        res
    }

    fn load_entry(&mut self) -> usize {
        let entry = u16::from_be_bytes(self.read_several::<2>());
        self.check_header("Entry loaded, start of functions");
        entry as usize
    }

    fn load_metadata(&mut self) {
        let tm = self.read_metadata_block("Types metadata");
        self.metadata.fill_types_metadata(tm);

        let lm = self.read_metadata_block("Lists metadata");
        self.metadata.fill_lists_metadata(lm);

        let fm = self.read_metadata_block("Functions metadata");
        let functions_count = fm.len();
        self.metadata.fill_function_metadata(fm);

        for i in 0..functions_count {
            let pos = u16::from_be_bytes(self.read_several::<2>()) as usize;
            self.metadata.function_positions.insert(pos, i);
        }
        self.check_header("End of function positions");
    }

    pub fn run(&mut self, step_by_step: bool, show_debug: bool) {
        self.check_header("Initial header");
        self.load_consts();
        if show_debug {
            println!("Loaded constants: {:?}", self.constants);
        }

        self.load_metadata();

        let entry = self.load_entry();
        self.call_op(entry, 0, 0);

        while self.ip < self.program.len() {
            if show_debug {
                println!(">> preparing to exec pc: {:02x?}", self.ip);
            }
            if step_by_step {
                io::stdin().read_line(&mut String::from("")).unwrap();
            }

            let opcode = self.read_opcode();
            match opcode {
                op::LOAD_CONST => {
                    let index = self.read_opcode();
                    push!(self, self.constants[index as usize]);
                }
                op::LOAD_SMALL_INT => {
                    let value = self.read_opcode();
                    push!(self, value as u64);
                }
                op::LOAD_TRUE => {
                    push!(self, 1);
                }
                op::LOAD_FALSE => {
                    push!(self, 0);
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
                op::GREATER_FLOAT => {
                    self.exec_binaryop(|a, b| (u64_to_f64(a) > u64_to_f64(b)) as u64)
                }
                op::LESS_FLOAT => self.exec_binaryop(|a, b| ((a as f64) < (b as f64)) as u64),
                op::EQ_FLOAT => self.exec_binaryop(|a, b| ((a as f64) == (b as f64)) as u64),

                // TODO: test bool operators
                op::NEGATE_BOOL => self.exec_unaryop(|x| x ^ 1),
                op::AND_BOOL => self.exec_binaryop(|a, b| a & b),
                op::OR_BOOL => self.exec_binaryop(|a, b| a | b),
                op::EQ_BOOL => self.exec_binaryop(|a, b| (a ^ b) ^ 1),

                op::ADD_STRINGS => {
                    let (b, a) = (self.pop(), self.pop());
                    let s1 = self.memory.get(a).extract_string();
                    let s2 = self.memory.get(b).extract_string();

                    let mut new_string = String::with_capacity(s1.len() + s2.len());
                    new_string.extend(s1.chars());
                    new_string.extend(s2.chars());

                    let (pos, _) = self.memory.move_string(new_string);
                    push!(self, pos);
                }
                op::EQ_STRINGS => {
                    let (b, a) = (self.pop(), self.pop());
                    let s1 = self.memory.get(a).extract_string();
                    let s2 = self.memory.get(b).extract_string();

                    let res = (s1 == s2) as u64;
                    push!(self, res);
                }

                op::GET_LOCAL => {
                    let value_pos = self.read_opcode() as usize;
                    let value_size = self.read_opcode() as usize;

                    let total_offset = self.current_frame().stack_start + value_pos;
                    for i in 0..value_size {
                        push!(self, self.stack[total_offset + i]);
                    }
                }
                op::SET_LOCAL => {
                    let value_pos = self.read_opcode() as usize;
                    let value_size = self.read_opcode() as usize;
                    // Go backwards because pop() returns items in a reversed order
                    for i in 0..value_size {
                        let value = self.pop();
                        self.stack
                            [self.current_frame().stack_start + value_pos + value_size - i - 1] =
                            value;
                    }
                }
                op::GET_OBJ_FIELD => {
                    let pointer = self.pop();
                    let offset = self.read_opcode() as usize;
                    let size = self.read_opcode() as usize;

                    let heap_obj = self.memory.get_mut(pointer);
                    let custom = heap_obj.extract_custom_object();

                    for value in custom.data.iter().skip(offset).take(size) {
                        push!(self, *value);
                    }
                }
                op::SET_OBJ_FIELD => {
                    let pointer = self.pop();
                    let offset = self.read_opcode() as usize;
                    let size = self.read_opcode() as usize;

                    let heap_obj = self.memory.get_mut(pointer);
                    let custom = heap_obj.extract_custom_object();

                    for i in 0..size {
                        let x = self.stack[self.stack_pointer - size + i];
                        custom.data[offset + i] = x;
                    }
                    self.stack_pointer -= size;
                }
                op::GET_LIST_ITEM => {
                    let list_pointer = self.pop();
                    let index = self.pop() as i64;

                    let heap_obj = self.memory.get_mut(list_pointer);
                    let list = heap_obj.extract_list();

                    let index = list.normalize_index(index);
                    let item_size = list.item_size;
                    let item_memory = list.get_item_mem(index);

                    for item in item_memory.iter().take(item_size) {
                        push!(self, *item);
                    }
                }
                op::SET_LIST_ITEM => {
                    let inner_offset = self.read_opcode() as usize; // offset per single element
                    let value_size = self.read_opcode() as usize;

                    let list_pointer = self.pop();
                    let index = self.pop() as i64;

                    let heap_obj = self.memory.get_mut(list_pointer);
                    let list = heap_obj.extract_list();

                    let index = list.normalize_index(index);
                    let memory_to_write = list.get_item_mem(index as usize);

                    self.stack_pointer -= value_size;
                    for i in 0..value_size {
                        memory_to_write[inner_offset + i] = self.stack[self.stack_pointer + i];
                    }
                }
                op::GET_TUPLE_ITEM => {
                    let tuple_size = self.read_opcode() as usize;
                    let offset = self.read_opcode() as usize;
                    let size_to_copy = self.read_opcode() as usize;
                    self.stack_pointer -= tuple_size;
                    for i in 0..size_to_copy {
                        let v = self.stack[self.stack_pointer + offset + i];
                        self.stack[self.stack_pointer - size_to_copy + i] = v;
                    }
                }
                op::ALLOCATE => {
                    let type_index = self.read_opcode() as usize;
                    let (new_obj_pos, _) = self.memory.allocate_custom(type_index, self.metadata);
                    push!(self, new_obj_pos);
                }
                op::ALLOCATE_LIST => {
                    let list_type_index = self.read_opcode() as usize;
                    let item_size = self.metadata.list_types_sizes[list_type_index] as usize;
                    let initial_items_amount = self.read_opcode() as usize;

                    self.stack_pointer -= item_size * initial_items_amount;

                    let (new_obj_pos, _) = self.memory.allocate_list(
                        list_type_index,
                        initial_items_amount,
                        &self.stack[self.stack_pointer..],
                        self.metadata
                    );
                    push!(self, new_obj_pos);
                }
                op::RESERVE => {
                    let value = self.read_opcode() as usize;

                    // TODO: check if performance is increased when reserve does not fills with 0
                    // If yes, probably worth giving some kind of flag
                    // this will make GC less precise, but
                    self.stack[self.stack_pointer..self.stack_pointer + value].fill(0);
                    self.stack_pointer += value;
                }
                op::CALL | op::CALL_STD => {
                    let return_size = self.read_opcode() as usize;
                    let args_size = self.read_opcode() as usize;
                    let function_pos = u16::from_be_bytes(self.read_several::<2>()) as usize;

                    match opcode {
                        op::CALL => self.call_op(function_pos, return_size, args_size),
                        op::CALL_STD => self.call_std(function_pos, return_size, args_size),
                        _ => unreachable!(),
                    }
                }
                op::POP => {
                    let amount = self.read_opcode();
                    for _ in 0..amount {
                        self.pop();
                    }
                }
                op::RETURN => {
                    self.return_op();
                    if self.frames.is_empty() {
                        if show_debug {
                            println!("VM finished!");
                        }
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
            if show_debug {
                println!(" ## STACK: {:02x?}", &self.stack[0..self.stack_pointer]);
                println!(" ## {}", &self.memory.simple_debug_view());
            }
        }
    }
}
