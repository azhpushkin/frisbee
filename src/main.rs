use std::path::Path;

pub mod ast;
pub mod types;
pub mod codegen;
pub mod errors;
pub mod loader;
pub mod parser;
pub mod semantics;
pub mod test_utils;
pub mod vm;

// TODO: color output?

fn main() {
    let file_path_s = std::env::args().last().unwrap();
    let file_path = Path::new(&file_path_s);
    if !file_path.is_file() {
        println!("{} is not a file!", file_path_s);
    }

    let mut wp = loader::load_program(file_path).expect("Error loading!");

    semantics::add_default_constructors(&mut wp);

    let aggregate = semantics::perform_semantic_analysis(&wp);

    let bytecode = codegen::generate(&aggregate);
    // let aux = codegen::disassemble::get_auxilary_functions_positions(&aggregate);

    // println!(
    //     "{}",
    //     codegen::disassemble::disassemble_bytes(&bytecode, Some(&aux))
    // );

    if true {
        let mut vm = vm::Vm::new(bytecode);
        vm.run();
    }
}
