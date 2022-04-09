use crate::loader::WholeProgram;

mod constants;
mod disassemble;
mod expressions;
mod functions;
mod generator;
mod globals;

pub fn generate_program(wp: &WholeProgram) -> Vec<u8> {
    let mut globals = globals::Globals::new();

    for (_, file) in wp.files.iter() {
        for function in file.ast.functions.iter() {
            let x = functions::generate_function_bytecode(function, &mut globals).unwrap();
            println!(
                "{}: \n{}",
                function.name,
                disassemble::disassemble_bytes(&x)
            )
        }
    }

    let const_bytecode = constants::constants_to_bytecode(&globals.constants);

    const_bytecode
}
