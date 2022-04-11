use std::path::Path;

pub mod ast;
// pub mod codegen;
pub mod errors;
pub mod loader;
pub mod parser;
pub mod semantics;
pub mod test_utils;
pub mod utils;
// pub mod vm;

// TODO: color output?

fn main() {
    let file_path_s = std::env::args().last().unwrap();
    let file_path = Path::new(&file_path_s);
    if !file_path.is_file() {
        println!("{} is not a file!", file_path_s);
    }

    let mut wp = loader::load_program(file_path).expect("Error loading!");

    let aggregate = semantics::perform_semantic_analysis(&wp);
    println!("{:?}", aggregate);
    // let symbols_info = semantic_checker::check_and_annotate_symbols(&mut wp).expect("Type error");
    // semantic_checker::check_and_annotate_statements(&mut wp, &symbols_info)
    //     .expect("Expr type error");

    // let bytecode = codegen::generate_program(&wp);
    // println!("{}", codegen::disassemble::disassemble_bytes(&bytecode));

    // if false {
    //     let mut vm = vm::Vm::new(bytecode);
    //     vm.run();
    // }
}
