use std::path::Path;

pub mod alias;
pub mod ast;
pub mod codegen;
pub mod errors;
pub mod loader;
pub mod parser;
pub mod semantics;
pub mod stdlib;
pub mod symbols;
pub mod test_utils;
pub mod types;
pub mod verified_ast;
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

    let mut wp = loader::load_program(file_path).unwrap_or_else(|(alias, source, error)| {
        errors::show_error_in_file(&alias, &source, error);
        panic!("See the error above!");
    });

    semantics::add_default_constructors(
        wp.files
            .iter_mut()
            .flat_map(|(_, loaded_file)| loaded_file.ast.types.iter_mut()),
    );
    let modules: Vec<_> = wp.iter().collect();

    let aggregate = semantics::perform_semantic_analysis(&modules, &wp.main_module).unwrap_or_else(
        |(alias, error)| {
            errors::show_error_in_file(&alias, &wp.files[&alias].contents, Box::new(error));
            panic!("See the error above!");
        },
    );

    let types: Vec<_> = aggregate.types.iter().map(|(_, value)| value).collect();
    let functions: Vec<_> = aggregate.functions.iter().map(|(_, value)| value).collect();
    let bytecode = codegen::generate(&types, &functions, &aggregate.entry);

    println!("{}", codegen::disassemble(&bytecode));

    let mut vm = vm::Vm::new(bytecode);
    vm.run(stepbystep, show_debug);
}
