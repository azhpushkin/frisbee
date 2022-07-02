use std::fs::File;
use std::io::Write;
use std::path::Path;

use argh::FromArgs;
use frisbee::os_loader;
use owo_colors::OwoColorize;

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

    #[argh(switch, short = 'r')]
    /// run immediately after compiling
    run: bool,
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
    let CompileCommand { mainfile, show_intermediate, run } = c;
    let (loader, main_module) = os_loader::entry_path_to_loader_and_main_module(&mainfile);

    let bytecode = frisbee::compile_program(&loader, main_module.clone());

    if show_intermediate {
        frisbee::show_intermediate(&loader, main_module, &mut std::io::stdout())
    }

    let bytecode_path = Path::new(&mainfile).with_extension("frisbee.bytecode");
    let mut bytecode_file = File::create(bytecode_path).expect("Cant open file for writing");
    bytecode_file.write_all(&bytecode).expect("Cant write to file");

    println!("{}", "File compiled successfully!".green());
    if run {
        frisbee::run_bytecode(bytecode)
    }
}

fn dis_file(c: DisCommand) {
    let DisCommand { program } = c;

    // xxd is also usefull way to show something inside of the file
    let bytecode = std::fs::read(program).expect("Cant read file");
    println!("{}", frisbee::disassemble_bytecode(&bytecode));
}

fn run_file(c: RunCommand) {
    let RunCommand { program, show_debug_info, step_by_step } = c;

    let bytecode = std::fs::read(program).expect("Cant read file");
    frisbee::run_bytecode(bytecode)
}

fn main() {
    // TODO: exit codes?
    let args: TopLevel = argh::from_env();
    match args.nested {
        FrisbeeSubCommands::Cc(c) => compile_file(c),
        FrisbeeSubCommands::Dis(c) => dis_file(c),
        FrisbeeSubCommands::Run(c) => run_file(c),
    };
}
