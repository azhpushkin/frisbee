use std::path::Path;

pub mod ast;
pub mod codegen;
pub mod errors;
pub mod loader;
pub mod parser;
pub mod semantics;
pub mod stdlib;
pub mod test_utils;
pub mod types;
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

    let aggregate = semantics::perform_semantic_analysis(&wp).expect("Error generating bytecode!");
    println!("{:#?}", aggregate);

    let bytecode = codegen::generate(&aggregate);

    println!("{}", codegen::disassemble(&bytecode));

    if true {
        let mut vm = vm::Vm::new(bytecode);
        vm.run();
    }
}
