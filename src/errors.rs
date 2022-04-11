use crate::{ast::ModulePath, parser, utils};

fn show_error(contents: &String, module: &ModulePath, pos: i32, error_msg: String) {
    let (line, row) = utils::get_position_coordinates(&contents, pos);

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
