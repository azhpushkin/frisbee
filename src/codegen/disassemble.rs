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
        _ => "<UNKNOWN>"
    }
}

pub fn disassemble_bytes(program: &Vec<u8>) -> String {
    let mut text_repr: String = String::new();

    let mut program_iter = program.iter();
    while let Some(opcode) = program_iter.next() {
        let mut args_num: usize = match *opcode {
            Op::LOAD_INT => 1,
            Op::LOAD_CONST => 1,
            Op::ADD_INT | Op::SUB_INT | Op::MUL_INT | Op::DIV_INT => 0,
            Op::ADD_FLOAT | Op::SUB_FLOAT | Op::MUL_FLOAT | Op::DIV_FLOAT => 0,
            Op::CALL => 1,

            _ => panic!("Unknown lol {}", opcode),
        };
        let mut args: Vec<u8> = vec![];
        while args_num > 0 {
            args.push(*program_iter.next().unwrap());
            args_num -= 1;
        }
        let op_text = format!("{:?} {:?}", opcode, args);
        text_repr.push_str(op_text.as_str());
    }
    text_repr
}
