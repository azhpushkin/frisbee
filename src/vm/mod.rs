mod engine;
mod heap;
mod metadata;
pub mod opcodes;
pub mod stdlib_runners;
mod utils;

pub type Vm = engine::Vm;
