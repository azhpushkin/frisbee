use std::slice::Iter;

use crate::vm::Op;

pub fn opcode_to_s(c: u8) -> &'static str {
    match c {
        Op::LOAD_CONST => "load_const",
        Op::LOAD_INT => "load_int",
        Op::ADD_INT => "add_int",
        Op::SUB_INT => "sub_int",
        Op::MUL_INT => "mul_int",
        Op::DIV_INT => "div_int",
        Op::ADD_FLOAT => "add_float",
        Op::SUB_FLOAT => "sub_float",
        Op::MUL_FLOAT => "mul_float",
        Op::DIV_FLOAT => "div_float",
        Op::CALL => "call",
        Op::RETURN => "return",
        Op::POP => "pop",
        Op::SET_VAR => "set_var",
        Op::GET_VAR => "get_var",
        _ => panic!("DIS: unknown opcode {}", c),
    }
}

pub fn get_str(i: &mut Iter<u8>, n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push(*i.next().unwrap() as char);
    }
    s
}
pub fn get_bytes<const N: usize>(i: &mut Iter<u8>) -> [u8; N] {
    let mut bytes = [0; N];
    for j in 0..N {
        bytes[j] = *i.next().unwrap();
    }
    bytes
}

pub fn disassemble_bytes(program: &Vec<u8>) -> String {
    let mut text_repr: String = String::from("Constants:\n");
    let mut program_iter = program.iter();

    loop {
        let mut i: usize = 0;
        // println!("{:?} {:?}", text_repr, *program_iter.next().unwrap());
        let const_text: String = match *program_iter.next().unwrap() {
            Op::CONST_INT_FLAG => i64::from_be_bytes(get_bytes::<8>(&mut program_iter)).to_string(),
            Op::CONST_FLOAT_FLAG => {
                f64::from_be_bytes(get_bytes::<8>(&mut program_iter)).to_string()
            }
            Op::CONST_STRING_FLAG => {
                let n = u16::from_be_bytes(get_bytes::<2>(&mut program_iter));
                get_str(&mut program_iter, n as usize)
            }
            Op::CONST_END_FLAG => break,
            c => panic!("Unknown const flag: {:02x}", c),
        };
        text_repr.push_str(&format!("\t{}: {}\n", i, const_text));
        i += 1;
    }
    return text_repr;

    while let Some(opcode) = program_iter.next() {
        let mut args_num: usize = match *opcode {
            Op::LOAD_INT => 1,
            Op::LOAD_CONST => 1,
            Op::ADD_INT | Op::SUB_INT | Op::MUL_INT | Op::DIV_INT => 0,
            Op::ADD_FLOAT | Op::SUB_FLOAT | Op::MUL_FLOAT | Op::DIV_FLOAT => 0,
            Op::CALL => 1,
            Op::RETURN => 0,
            Op::POP => 0,
            Op::SET_VAR | Op::GET_VAR => 1,
            _ => panic!("DIS: Unknown lol {}", opcode),
        };
        let mut args: Vec<u8> = vec![];
        while args_num > 0 {
            args.push(*program_iter.next().unwrap());
            args_num -= 1;
        }
        let op_text = format!("\t{} {:02x?}\n", opcode_to_s(*opcode), args);
        text_repr.push_str(op_text.as_str());
    }
    text_repr
}
