pub mod ast;
pub mod errors;
pub mod loader;
pub mod parser;
pub mod scanner;
pub mod semantic_checker;
pub mod test_utils;
#[allow(dead_code)]
pub mod tree_walk; // TODO: remove
pub mod utils;

// TODO: color output?

use std::path::Path;

fn main() {
    let file_path_s = std::env::args().last().unwrap();
    let file_path = Path::new(&file_path_s);
    if !file_path.is_file() {
        println!("{} is not a file!", file_path_s);
    }

    loader::load_program(file_path);
}
