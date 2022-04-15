use crate::{ast::ModulePath, parser};

pub fn get_position_coordinates(data: &String, pos: i32) -> (usize, usize) {
    let mut line: usize = 0;
    let mut row: usize = 0;
    let mut counter: i32 = 0;

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


fn show_error(contents: &String, module: &ModulePath, pos: i32, error_msg: String) {
    let (line, row) = get_position_coordinates(&contents, pos);

    println!(
        "Error at line {} (in {}):\n----------\n",
        line,
        module.alias().0
    );

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
    module: &ModulePath,
    error: parser::scanner::ScanningError,
) {
    show_error(contents, module, error.1, error.0.into());
}

pub fn show_parse_error(contents: &String, module: &ModulePath, error: parser::ParseError) {
    let formatted_error_msg = match error.expected {
        Some(token) => format!("{} (Expected token <{:?}>)", error.error_msg, token),
        None => error.error_msg.to_string(),
    };

    show_error(contents, module, error.error_at.1, formatted_error_msg);
}
