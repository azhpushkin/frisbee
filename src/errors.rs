use crate::alias::ModuleAlias;
use crate::parser;

pub fn get_position_coordinates(data: &String, pos: usize) -> (usize, usize) {
    let mut line: usize = 0;
    let mut row: usize = 0;
    let mut counter: usize = 0;

    let mut chars = data.chars();
    while counter < pos {
        if chars.next().unwrap() == '\n' {
            line += 1;
            row = 0;
        } else {
            row += 1;
        }

        counter += 1;
    }
    (line, row)
}

fn show_error(contents: &String, alias: &ModuleAlias, pos: usize, error_msg: String) {
    let (line, row) = get_position_coordinates(&contents, pos);

    println!("Error at line {} (in {}):\n----------\n", line, alias);

    let lines: Vec<&str> = contents.split('\n').collect();
    let spaces: String = vec![' '; row].into_iter().collect();

    // Print lines of code, 2 if possible
    println!(
        "{:?}\n{}",
        if line > 0 { lines[line - 1] } else { "" },
        lines[line]
    );
    // Print pointer to error and error inself
    println!("{}^\n{}{}", spaces, spaces, error_msg);
}

pub fn show_scan_error(
    contents: &String,
    alias: &ModuleAlias,
    error: parser::scanner::ScanningError,
) {
    show_error(contents, alias, error.1, error.0.into());
}

pub fn show_parse_error(contents: &String, alias: &ModuleAlias, error: parser::ParseError) {
    let formatted_error_msg = match error.expected {
        Some(token) => format!("{} (Expected token <{:?}>)", error.error_msg, token),
        None => error.error_msg.to_string(),
    };
    // TODO: show error highlighted?
    show_error(contents, alias, error.error_at.first, formatted_error_msg);
}
