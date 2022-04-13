use std::iter::Enumerate;
use std::slice::Iter;

use crate::vm::opcodes::{get_args_num, op};

pub fn opcode_to_s(c: u8) -> &'static str {
    match c {
        op::LOAD_CONST => "load_const",
        op::LOAD_INT => "load_int",
        op::LOAD_TRUE => "load_true",
        op::LOAD_FALSE => "load_false",
        op::ADD_INT => "add_int",
        op::SUB_INT => "sub_int",
        op::MUL_INT => "mul_int",
        op::DIV_INT => "div_int",
        op::ADD_FLOAT => "add_float",
        op::SUB_FLOAT => "sub_float",
        op::MUL_FLOAT => "mul_float",
        op::DIV_FLOAT => "div_float",
        op::CALL => "call",
        op::RETURN => "return",
        op::POP => "pop",
        op::SET_VAR => "set_var",
        op::GET_VAR => "get_var",
        _ => panic!("DIS: unknown opcode {}", c),
    }
}

pub fn get_str(i: &mut Enumerate<Iter<u8>>) -> String {
    let n = u16::from_be_bytes(get_bytes::<2>(i));
    let mut s = String::new();
    for _ in 0..n {
        s.push(*i.next().unwrap().1 as char);
    }
    s
}
pub fn get_bytes<const N: usize>(i: &mut Enumerate<Iter<u8>>) -> [u8; N] {
    let mut bytes = [0; N];
    for j in 0..N {
        bytes[j] = *i.next().unwrap().1;
    }
    bytes
}

pub fn disassemble_bytes(program: &Vec<u8>) -> String {
    println!("Program is of total len: {}", program.len());

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
            op::CONST_END_FLAG => break,
            c => panic!("Unknown const flag: {:02x}", c),
        };
        text_repr.push_str(&format!("\t{}: {}\n", i, const_text));
        i += 1;
    }
    text_repr.push_str("\nFunctions:\n");

    while let Some((i, opcode)) = program_iter.next() {
        let mut number_of_args = get_args_num(*opcode);
        let mut args: Vec<u8> = vec![];
        while number_of_args > 0 {
            args.push(*program_iter.next().unwrap().1);
            number_of_args -= 1;
        }
        let op_text = format!("{:02x?}\t{} {:02x?}\n", i, opcode_to_s(*opcode), args);
        text_repr.push_str(op_text.as_str());
    }
    text_repr
}
