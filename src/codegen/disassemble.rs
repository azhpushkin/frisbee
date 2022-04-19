use std::collections::HashMap;
use std::iter::Enumerate;
use std::slice::Iter;

use crate::semantics::aggregate::ProgramAggregate;
use crate::semantics::symbols::SymbolFunc;
use crate::vm::opcodes::{get_args_num, op};

use super::generate_chunks;

pub fn opcode_to_s(c: u8) -> &'static str {
    match c {
        op::RESERVE_ONE => "reserve_one",
        op::LOAD_CONST => "load_const",
        op::LOAD_INT => "load_int",
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

        op::ADD_FLOAT => "add_float",
        op::SUB_FLOAT => "sub_float",
        op::MUL_FLOAT => "mul_float",
        op::DIV_FLOAT => "div_float",
        op::CALL => "call",
        op::RETURN => "return",
        op::POP => "pop",
        op::SET_VAR => "set_var",
        op::GET_VAR => "get_var",

        op::JUMP => "jump",
        op::JUMP_BACK => "jump_back",
        op::JUMP_IF_FALSE => "jump_if_false",

        _ => panic!("DIS: unknown opcode {}", c),
    }
}

pub struct Disassembler<'a> {
    program: &'a Vec<u8>,
    program_iter: Box<dyn Iterator<Item = (usize, &'a u8)> + 'a>,
}


impl<'a> Disassembler<'a> {
    pub fn new(program: &'a Vec<u8>) -> Self {
        Disassembler {
            program,
            program_iter: Box::new(program.iter().enumerate()),
        }
    }

    fn get_str(&mut self) -> String {
        let n = u16::from_be_bytes(get_bytes::<2>());
        let mut s = String::new();
        for _ in 0..n {
            s.push(*self.program_iter.next().unwrap().1 as char);
        }
        s
    }

    fn get_bytes<const N: usize>(&mut self) -> [u8; N] {
        let mut bytes = [0; N];
        for j in 0..N {
            bytes[j] = *self.program_iter.next().unwrap().1;
        }
        bytes
    }

    fn disassemble_bytes(program: &Vec<u8>, aux: Option<&AuxData>) -> String {
        let mut text_repr: String = String::from("Constants:\n");
        let mut program_iter = program.iter().enumerate();
    
        let mut i: usize = 0;
        loop {
            let const_text: String = match *program_iter.next().unwrap().1 {
                op::CONST_INT_FLAG => i64::from_be_bytes(get_bytes::<8>(&mut program_iter)).to_string(),
                op::CONST_FLOAT_FLAG => {
                    f64::from_be_bytes(get_bytes::<8>(&mut program_iter)).to_string()
                }
                op::CONST_STRING_FLAG => get_str(&mut program_iter),
                op::CONST_END_FLAG => {
                    let x = u16::from_be_bytes(get_bytes::<2>(&mut program_iter));
                    text_repr.push_str(&format!("Entry: {:02x}", x));
                    break;
                }
                c => panic!("Unknown const flag: {:02x}", c),
            };
            text_repr.push_str(&format!("\t{}: {}\n", i, const_text));
            i += 1;
        }
        text_repr.push_str("\nFunctions:\n");
    
        while let Some((i, opcode)) = program_iter.next() {
            if aux.is_some() && aux.unwrap().get(&i).is_some() {
                let func = aux.unwrap().get(&i).unwrap();
                text_repr.push_str(&format!("\n# {:?}:\n", func));
            }
    
            let mut number_of_args = get_args_num(*opcode);
            let mut args: Vec<u8> = vec![];
            while number_of_args > 0 {
                args.push(*program_iter.next().unwrap().1);
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
    
            text_repr.push_str(op_text.as_str());
            text_repr.push_str("\n");
        }
        text_repr
    }


}

