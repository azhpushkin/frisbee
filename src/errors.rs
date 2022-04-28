use crate::alias::ModuleAlias;
use crate::parser::scanner::ScanningError;
use crate::parser::ParseError;
use crate::semantics::errors::SemanticError;

pub trait CompileError {
    fn get_position_window(&self) -> (usize, usize);
    fn get_message(&self) -> String;
}

impl CompileError for ScanningError {
    fn get_position_window(self: &ScanningError) -> (usize, usize) {
        (self.1, self.1)
    }

    fn get_message(&self) -> String {
        format!("Scanning error: {}", self.0)
    }
}

impl CompileError for ParseError {
    fn get_position_window(self: &ParseError) -> (usize, usize) {
        (self.error_at.first, self.error_at.last)
    }

    fn get_message(&self) -> String {
        match self.expected {
            Some(token) => format!("{} (Expected token <{:?}>)", self.error_msg, token),
            None => self.error_msg.to_string(),
        }
    }
}

impl CompileError for SemanticError {
    fn get_position_window(self: &SemanticError) -> (usize, usize) {
        match self {
            SemanticError::ExprError { pos_first, pos_last, .. } => (*pos_first, *pos_last),
            SemanticError::StmtError { pos, .. } => (*pos, *pos),
            SemanticError::TopLevelError { .. } => (0, 0),
        }
    }

    fn get_message(&self) -> String {
        match self {
            SemanticError::ExprError { message, .. } => message.clone(),
            SemanticError::StmtError { message, .. } => message.clone(),
            SemanticError::TopLevelError { message } => message.clone(),
        }
    }
}

struct ErrorCoordinates {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

fn get_lines_length(source: &str) -> Vec<usize> {
    let mut lines_length = vec![];
    let mut current_line_length = 0;
    for c in source.chars() {
        if c == '\n' {
            lines_length.push(current_line_length);
            current_line_length = 0;
        } else {
            current_line_length += 1;
        }
    }
    lines_length.push(current_line_length);
    lines_length
}

fn get_position_coordinates(file_contents: &str, pos: usize) -> (usize, usize) {
    let mut line: usize = 0;
    let mut row: usize = 0;
    let mut counter: usize = 0;

    let mut chars = file_contents.chars();
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

fn adjust_error_window(source: &str, start: usize, end: usize) -> ErrorCoordinates {
    let lines_length = get_lines_length(source);
    let (start_line, start_offset) = get_position_coordinates(source, start);
    if start == end {
        return ErrorCoordinates {
            line: start_line,
            start: start_offset,
            end: lines_length[start_line],
        };
    }
    let (end_line, end_offset) = get_position_coordinates(source, end);

    let line = start_line;
    let start = start_offset;
    let end = if start_line != end_line {
        lines_length[start_line]
    } else {
        end_offset + 1
    };
    ErrorCoordinates { line: start_line, start, end }
}

fn show_error(contents: &String, alias: &ModuleAlias, pos: ErrorCoordinates, error_msg: String) {
    // TODO: add path here somehow, maybe just pass as an argument...
    println!("Error at line {} (in {}):\n----------\n", pos.line, alias);

    let lines: Vec<&str> = contents.split('\n').collect();
    let spaces: String = vec![' '; pos.start].into_iter().collect();
    let underscored: String = vec!['~'; pos.end - pos.start].into_iter().collect();

    // Print lines of code, 2 if possible

    for i in (pos.line - 2).max(0)..pos.line {
        println!("{}", lines[i]);
    }
    println!("{}^{}\n{}{}\n", spaces, underscored, spaces, error_msg);
}

pub fn show_error_in_file<E>(alias: &ModuleAlias, source: &String, error: &E)
where
    E: CompileError,
{
    let (start, end) = error.get_position_window();
    let error_window = adjust_error_window(&source, start, end);
    let error_msg = error.get_message();

    show_error(&source, alias, error_window, error_msg);
}
