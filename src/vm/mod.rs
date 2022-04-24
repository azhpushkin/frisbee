mod vm;
pub mod stdlib_runners;
pub mod opcodes;
mod utils;
mod heap;

pub type Vm = vm::Vm;