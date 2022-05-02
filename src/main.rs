use argh::FromArgs;
use std::path::Path;

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

fn main() {
    let args: TopLevel = argh::from_env();
    match &args.nested {
        FrisbeeSubCommands::Cc(CompileCommand { mainfile, show_intermediate }) => {
            println!("compiling {}", mainfile);
        }
        FrisbeeSubCommands::Dis(DisCommand { program }) => {
            println!("Disassembling  {}", program);
        }
        FrisbeeSubCommands::Run(RunCommand { program, show_debug_info, step_by_step }) => {
            println!("Running {}", program);
        }
    }
    // let args: Vec<String> = std::env::args().collect();
    // let file_path_s: String;
    // let mut show_debug: bool = false;
    // let mut stepbystep: bool = false;

    // let last_arg = args.last().unwrap();
    // if last_arg.contains(".frisbee") {
    //     file_path_s = last_arg.clone();
    // } else {
    //     file_path_s = args[args.len() - 2].clone();
    //     show_debug = last_arg == "debug" || last_arg == "stepbystep";
    //     stepbystep = last_arg == "stepbystep";
    // }

    // let file_path = Path::new(&file_path_s);
    // if !file_path.is_file() {
    //     println!("{} is not a file!", file_path_s);
    // }

    // let mut wp = loader::load_program(file_path).unwrap_or_else(|(alias, source, error)| {
    //     errors::show_error_in_file(&alias, &source, error);
    //     panic!("See the error above!");
    // });

    // semantics::add_default_constructors(
    //     wp.files
    //         .iter_mut()
    //         .flat_map(|(_, loaded_file)| loaded_file.ast.types.iter_mut()),
    // );
    // let modules: Vec<_> = wp.iter().collect();

    // let aggregate =
    //     semantics::perform_semantic_analysis(&modules, &wp.main_module).unwrap_or_else(|err| {
    //         errors::show_error_in_file(
    //             &err.module,
    //             &wp.files[&err.module].contents,
    //             Box::new(err.error),
    //         );
    //         // return 1 exit code instead of panic
    //         panic!("See the error above!");
    //     });

    // let types: Vec<_> = aggregate.types.iter().map(|(_, value)| value).collect();
    // let functions: Vec<_> = aggregate.functions.iter().map(|(_, value)| value).collect();
    // let bytecode = codegen::generate(&types, &functions, &aggregate.entry);

    // println!("#####Verified:\n\n");
    // for (_, func) in aggregate.functions.iter() {
    //     println!("{}\n\n", func);
    // }

    // println!("{}", codegen::disassemble(&bytecode));

    // let mut vm = vm::Vm::new(bytecode);
    // vm.run(stepbystep, show_debug);
}
