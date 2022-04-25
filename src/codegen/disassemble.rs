use std::collections::HashMap;
use std::iter::Enumerate;
use std::slice::Iter;

use crate::semantics::aggregate::ProgramAggregate;
use crate::semantics::symbols::SymbolFunc;
use crate::vm::opcodes::{get_args_num, op};

use super::generate_chunks;

pub fn opcode_to_s(c: u8) -> &'static str {
    match c {
        op::RESERVE => "reserve",
        op::LOAD_CONST => "load_const",
        op::LOAD_SMALL_INT => "load_small_int",
        op::LOAD_TRUE => "load_true",
        op::LOAD_FALSE => "load_false",

        op::NEGATE_INT => "negate_int",
        op::ADD_INT => "add_int",
        op::SUB_INT => "sub_int",
        op::MUL_INT => "mul_int",
        op::DIV_INT => "div_int",
        op::GREATER_INT => "greater_int",
        op::LESS_INT => "less_int",
        op::EQ_INT => "eq_int",

        op::NEGATE_FLOAT => "negate_float",
        op::ADD_FLOAT => "add_float",
        op::SUB_FLOAT => "sub_float",
        op::MUL_FLOAT => "mul_float",
        op::DIV_FLOAT => "div_float",
        op::GREATER_FLOAT => "greater_float",
        op::LESS_FLOAT => "less_float",
        op::EQ_FLOAT => "eq_float",

        op::NEGATE_BOOL => "negate_bool",
        op::EQ_BOOL => "eq_bool",
        op::AND_BOOL => "and_bool",
        op::OR_BOOL => "or_bool",

        op::ADD_STRINGS => "add_strings",
        op::EQ_STRINGS => "eq_strings",

        op::CALL => "call",
        op::CALL_STD => "call_std",
        op::RETURN => "return",
        op::POP => "pop",
        op::ALLOCATE => "allocate",
        op::SET_LOCAL => "set_local",
        op::GET_LOCAL => "get_local",
        op::GET_TUPLE_ITEM => "get_tuple_item",

        op::JUMP => "jump",
        op::JUMP_BACK => "jump_back",
        op::JUMP_IF_FALSE => "jump_if_false",

        _ => panic!("DIS: unknown opcode {}", c),
    }
}

pub struct Disassembler<'a> {
    program_iter: Box<dyn Iterator<Item = (usize, &'a u8)> + 'a>,
    result: Vec<String>,
    symbol_names: HashMap<usize, String>,
}

impl<'a> Disassembler<'a> {
    pub fn new(program: &'a Vec<u8>) -> Self {
        Disassembler {
            program_iter: Box::new(program.iter().enumerate()),
            result: vec![],
            symbol_names: HashMap::new(),
        }
    }

    pub fn disassemble(&mut self) -> String {
        self.result.clear();

        self.read_header("Initial");
        self.read_constants();
        self.read_header("End of constants");
        self.read_symbol_names();
        self.read_header("End of symbol names");
        self.read_entry();
        self.read_header("Start of functions");
        self.read_functions();

        return self.result.join("\n");
    }

    fn get_byte(&mut self) -> (usize, u8) {
        let (i, byte) = self.program_iter.next().unwrap();
        (i, *byte)
    }

    fn get_str(&mut self) -> String {
        let n = u16::from_be_bytes(self.get_bytes::<2>());
        let mut s = String::new();
        for _ in 0..n {
            s.push(self.get_byte().1 as char);
        }
        s
    }

    fn get_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0; N];
        for j in 0..N {
            bytes[j] = self.get_byte().1;
        }
        bytes
    }

    fn read_header(&mut self, header_name: &str) {
        let header = self.get_bytes::<2>();
        self.result
            .push(format!("HEADER [{}] => {:02x?}", header_name, header));
    }

    fn read_constants(&mut self) {
        self.result.push(format!("Constants table:"));
        let mut i: usize = 0;

        loop {
            let const_text: String = match self.get_byte().1 {
                op::CONST_INT_FLAG => i64::from_be_bytes(self.get_bytes::<8>()).to_string(),
                op::CONST_FLOAT_FLAG => f64::from_be_bytes(self.get_bytes::<8>()).to_string(),
                op::CONST_STRING_FLAG => format!("\"{}\"", self.get_str()),
                op::CONST_END_FLAG => {
                    break;
                }
                c => panic!("Unknown const flag: {:02x}", c),
            };
            self.result.push(format!("   {} -> {}", i, const_text));
            i += 1;
        }
    }

    fn read_symbol_names(&mut self) {
        loop {
            let symbol_name = self.get_str();
            if symbol_name.len() == 0 {
                break;
            }

            let pos = u16::from_be_bytes(self.get_bytes::<2>()) as usize;
            self.symbol_names.insert(pos, symbol_name);
        }
    }

    fn read_entry(&mut self) {
        let entry = self.get_bytes::<2>();
        let entry_name = &self.symbol_names[&(u16::from_be_bytes(entry) as usize)];
        self.result
            .push(format!("Entry point:\n   -> {} {:02x?}", entry_name, entry));
    }

    fn read_functions(&mut self) {
        while let Some((i, opcode)) = self.program_iter.next() {
            if let Some(name) = self.symbol_names.get(&i) {
                self.result.push(format!("\n## Function {}", name));
            }

            let mut number_of_args = get_args_num(*opcode);
            let mut args: Vec<u8> = vec![];
            while number_of_args > 0 {
                args.push(self.get_byte().1);
                number_of_args -= 1;
            }
            let mut op_text = format!(" {:02x?}   {} {:02x?}", i, opcode_to_s(*opcode), args);

            // + 3 is added, because jump offset is relative to instruction pointer
            // but `i` var points to jump opcode, which is 3 steps behind
            // (1 step for jump itself and 2 for address of jump)
            if *opcode == op::JUMP_IF_FALSE || *opcode == op::JUMP {
                let x = u16::from_be_bytes([args[0], args[1]]) as usize;
                op_text = format!("{} (jumps to {:02x?}) ", op_text, x + i + 3);
            } else if *opcode == op::JUMP_BACK {
                let x = u16::from_be_bytes([args[0], args[1]]) as usize;
                op_text = format!("{} (jumps to {:02x?}) ", op_text, i - x + 3);
            }

            self.result.push(op_text);
        }
    }
}
