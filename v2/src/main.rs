pub mod ast;
pub mod parser;
pub mod tokens;

fn main() {
    let file_path = std::env::args().last().unwrap();
    println!("Loading {}", file_path);

    let file_contents = std::fs::read_to_string(file_path).expect("Cant read file");

    println!(
        "=====FILE LOADED=====\n{}\n=====================",
        file_contents
    );

    let tokens = tokens::scan_tokens(file_contents);

    let ast = parser::parse(tokens);

    println!("Ast is: {:?}", ast)
}
