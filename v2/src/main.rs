pub mod ast;
pub mod parser;
pub mod scanner;
pub mod semantic_checker;

// TODO: color output?

fn show_parse_error(program: &String, error: parser::ParseError) {
    let (line, row) = scanner::get_token_coordinates(&program, error.error_at);

    println!("Error at line {} (row {}):\n----------\n", line, row,);

    let lines: Vec<&str> = program.split('\n').collect();
    let spaces: String = vec![' '; row].into_iter().collect();
    let formatted_error_msg = match error.expected {
        Some(token) => format!("{} (Expected token <{}>)", error.error_msg, token),
        None => error.error_msg.to_string(),
    };

    // Print lines of code, 2 if possible
    println!(
        "{:?}\n{}",
        if line > 0 { lines[line - 1] } else { "" },
        lines[line]
    );
    // Print pointer to error and error inself
    println!("{}^\n{}{}", spaces, spaces, formatted_error_msg);
}

fn main() {
    let file_path = std::env::args().last().unwrap();
    println!(" ... Loading {}\n\n", file_path);

    let file_contents = std::fs::read_to_string(file_path).expect("Cant read file");

    let tokens = scanner::scan_tokens(&file_contents);

    let ast: parser::ParseResult<ast::Program> = parser::parse(tokens);

    if ast.is_err() {
        show_parse_error(&file_contents, ast.unwrap_err());
        return;
    }

    println!("Parsed to {:?}", ast.unwrap());
}
