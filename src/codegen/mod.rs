use crate::loader::WholeProgram;

mod disassemble;
mod expressions;
mod functions;
mod globals;

pub fn generate_program(wp: &WholeProgram) {
    let mut globals = globals::Globals::new();

    for (_, file) in wp.files.iter() {

        for function in file.ast.functions.iter() {
            let x = functions::generate_function_bytecode(function, &mut globals).unwrap();
            println!("{} bytes: {}", function.name, disassemble::disassemble_bytes(&x))
        }
    }
}