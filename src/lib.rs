use std::io;
use std::sync::{Arc, Mutex};

pub mod alias;
pub mod ast;
pub mod codegen;
pub mod errors;
pub mod loader;
pub mod os_loader;
pub mod parsing;
pub mod runtime;
pub mod semantics;
pub mod stdlib;
pub mod symbols;
pub mod tests;
pub mod types;

fn load_and_verify_program(
    loader: &dyn loader::FrisbeeModuleLoader,
    main_module: String,
) -> semantics::aggregate::ProgramAggregate {
    let main_alias = alias::ModuleAlias::new(&[main_module]);

    let mut wp = loader::load_modules_recursively(loader, &main_alias).unwrap_or_else(
        |(alias, source, error)| {
            errors::show_error_in_file(&alias, &source, error);
            panic!("See the error above!");
        },
    );

    semantics::add_default_constructors(
        wp.modules
            .iter_mut()
            .flat_map(|(_, loaded_file)| loaded_file.ast.types.iter_mut()),
    );
    let modules: Vec<_> = wp.iter().collect();

    let aggregate =
        semantics::perform_semantic_analysis(&modules, &wp.main_module).unwrap_or_else(|err| {
            errors::show_error_in_file(
                &err.module,
                &wp.modules[&err.module].contents,
                Box::new(err.error),
            );
            // return 1 exit code instead of panic
            panic!("See the error above!");
        });

    aggregate
}

pub fn compile_program(loader: &dyn loader::FrisbeeModuleLoader, main_module: String) -> Vec<u8> {
    let aggregate = load_and_verify_program(loader, main_module);

    let semantics::aggregate::ProgramAggregate { types, functions, entry } = aggregate;
    let types: Vec<_> = types.into_iter().map(|(_, v)| v).collect();
    let functions: Vec<_> = functions.into_iter().map(|(_, v)| v).collect();
    codegen::generate(&types, &functions, &entry)
}

pub fn show_intermediate(
    loader: &dyn loader::FrisbeeModuleLoader,
    main_module: String,
    output: &mut dyn io::Write,
) {
    let aggregate = load_and_verify_program(loader, main_module);

    for (_, func) in aggregate.functions.iter() {
        write!(output, "{}\n", func).unwrap();
    }
}

pub fn disassemble_bytecode(bytecode: &[u8]) -> String {
    codegen::disassemble(&bytecode)
}

pub fn run_bytecode(bytecode: Vec<u8>, output: Arc<Mutex<runtime::vm::Output>>) {
    let vm = runtime::vm::Vm::setup(bytecode, output);
    runtime::vm::Vm::setup_entry_and_run(vm)
}
