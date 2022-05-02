use std::fs::File;
use std::io::Write;
use std::path::Path;

use argh::FromArgs;
use owo_colors::OwoColorize;

pub mod alias;
pub mod ast;
pub mod codegen;
pub mod errors;
pub mod loader;
pub mod parsing;
pub mod semantics;
pub mod stdlib;
pub mod symbols;
pub mod tests;
pub mod types;
pub mod vm;

// TODO: color output?

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    nested: FrisbeeSubCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum FrisbeeSubCommands {
    Cc(CompileCommand),
    Dis(DisCommand),
    Run(RunCommand),
}

#[derive(FromArgs, PartialEq, Debug)]
/// Compile subcommand.
#[argh(subcommand, name = "cc")]
struct CompileCommand {
    #[argh(positional)]
    /// path to main compilation target
    mainfile: String,

    #[argh(switch, short = 'i')]
    /// show intermediate representation during compilation
    show_intermediate: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Disassemble subcommand.
#[argh(subcommand, name = "dis")]
struct DisCommand {
    #[argh(positional)]
    /// path to compiled program
    program: String,
}

#[derive(FromArgs, PartialEq, Debug)]
/// Run subcommand.
#[argh(subcommand, name = "run")]
struct RunCommand {
    #[argh(positional)]
    /// path to compiled program
    program: String,

    #[argh(switch, short = 'i')]
    /// show debug info on each tick
    show_debug_info: bool,

    #[argh(switch, short = 's')]
    /// execute step by step for debug porposes
    step_by_step: bool,
}

fn compile_file(c: CompileCommand) {
    let CompileCommand { mainfile, show_intermediate } = c;
    let file_path = Path::new(&mainfile);

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

    let aggregate =
        semantics::perform_semantic_analysis(&modules, &wp.main_module).unwrap_or_else(|err| {
            errors::show_error_in_file(
                &err.module,
                &wp.files[&err.module].contents,
                Box::new(err.error),
            );
            // return 1 exit code instead of panic
            panic!("See the error above!");
        });

    if show_intermediate {
        println!("#####Verified:\n\n");
        for (_, func) in aggregate.functions.iter() {
            println!("{}\n", func);
        }
    }

    let types: Vec<_> = aggregate.types.iter().map(|(_, value)| value).collect();
    let functions: Vec<_> = aggregate.functions.iter().map(|(_, value)| value).collect();
    let bytecode = codegen::generate(&types, &functions, &aggregate.entry);

    let bytecode_path = file_path.with_extension("frisbee.bytecode");
    let mut bytecode_file = File::create(bytecode_path).expect("Cant open file for writing");
    bytecode_file.write_all(&bytecode).expect("Cant write to file");

    println!("{}", "File compiled successfully!".green());
}

fn dis_file(c: DisCommand) {
    let DisCommand { program } = c;

    // xxd is also usefull way to show something inside of the file
    let bytecode = std::fs::read(program).expect("Cant read file");
    println!("{}", codegen::disassemble(&bytecode));
}

fn run_file(c: RunCommand) {
    let RunCommand { program, show_debug_info, step_by_step } = c;

    let bytecode = std::fs::read(program).expect("Cant read file");

    let mut vm = vm::Vm::new(bytecode);
    vm.run(step_by_step, show_debug_info);
}

fn main() {
    let args: TopLevel = argh::from_env();
    match args.nested {
        FrisbeeSubCommands::Cc(c) => compile_file(c),
        FrisbeeSubCommands::Dis(c) => dis_file(c),
        FrisbeeSubCommands::Run(c) => run_file(c),
    }
}
