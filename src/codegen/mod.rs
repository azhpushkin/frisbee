suse crate::vm::Op;

pub fn disassemble_bytes(program: Vec<u8>) -> String {
    let mut text_repr: String = String::new();

    let mut program_iter = program.iter();
    while let Some(opcode) = program_iter.next() {
        let mut args_num: usize = match opcode {
            &Op::LOAD_INT => 1,
            _ => 0,
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
