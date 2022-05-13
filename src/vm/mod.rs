mod engine;
mod heap;
mod metadata;
pub mod opcodes;
pub mod stdlib_runners;
mod utils;
mod serialization;

pub type Vm = engine::Vm;
