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
    let args: Vec<String> = std::env::args().collect();
    let file_path_s: String;
    let mut show_debug: bool = false;
    let mut stepbystep: bool = false;

    let last_arg = args.last().unwrap();
    if last_arg.contains(".frisbee") {
        file_path_s = last_arg.clone();
    } else {
        file_path_s = args[args.len() - 2].clone();
        show_debug = last_arg == "debug" || last_arg == "stepbystep";
        stepbystep = last_arg == "stepbystep";
    }

    let file_path = Path::new(&file_path_s);
    if !file_path.is_file() {
        println!("{} is not a file!", file_path_s);
    }

    let mut wp = loader::load_program(file_path).expect("Error loading!");

    semantics::add_default_constructors(&mut wp);

    let aggregate = semantics::perform_semantic_analysis(&wp).expect("Error generating bytecode!");
    // println!("{:#?}", aggregate);

    let bytecode = codegen::generate(&aggregate);

    println!("{}", codegen::disassemble(&bytecode));

    let mut vm = vm::Vm::new(bytecode);
    vm.run(stepbystep, show_debug);
}
