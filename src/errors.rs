use crate::alias::ModuleAlias;
use crate::parsing::scanner::ScanningError;
use crate::parsing::ParseError;
use crate::semantics::errors::SemanticError;

use std::io::{self, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub trait CompileError: std::fmt::Debug {
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
        match &self.expected {
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
            SemanticError::TopLevelError { pos, .. } => (*pos, *pos),
        }
    }

    fn get_message(&self) -> String {
        match self {
            SemanticError::ExprError { message, .. } => message.clone(),
            SemanticError::StmtError { message, .. } => message.clone(),
            SemanticError::TopLevelError { message, .. } => message.clone(),
        }
    }
}

#[derive(Debug)]
pub struct ErrorCoordinates {
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

pub fn adjust_error_window(source: &str, start: usize, end: usize) -> ErrorCoordinates {
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

    let end = if start_line != end_line {
        lines_length[start_line]
    } else {
        end_offset + 1
    };
    ErrorCoordinates { line: start_line, start: start_offset, end }
}

fn show_error(
    contents: &String,
    alias: &ModuleAlias,
    pos: ErrorCoordinates,
    error_msg: String,
) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let no_color = |stdout: &mut StandardStream| stdout.set_color(ColorSpec::new().set_fg(None));
    let set_color =
        |stdout: &mut StandardStream, o: Color| stdout.set_color(ColorSpec::new().set_fg(Some(o)));

    // TODO: add path here somehow, maybe just pass as an argument...
    let header = format!("Error at line {} (in {}):", pos.line, alias);
    set_color(&mut stdout, Color::Red)?;
    writeln!(&mut stdout, "{}", header)?;

    let header_underscore = vec!["="; header.len() - 1];
    no_color(&mut stdout)?;
    writeln!(&mut stdout, "{}\n", header_underscore.join(""))?;

    let sidebar_len = pos.line.to_string().len() + 3; // "<num> | " - 3 chars + line len

    let lines: Vec<&str> = contents.split('\n').collect();
    let spaces: String = vec![' '; pos.start + sidebar_len].into_iter().collect();
    let underscored: String = vec!['~'; pos.end - pos.start].into_iter().collect();

    // Print lines of code, 2 if possible

    let first_list = (pos.line as i64 - 2).max(0) as usize;
    for line in first_list..pos.line + 1 {
        set_color(&mut stdout, Color::Blue)?;
        write!(&mut stdout, "{} | ", line)?;
        no_color(&mut stdout)?;
        writeln!(&mut stdout, "{}", lines[line])?;
    }
    set_color(&mut stdout, Color::Yellow)?;
    writeln!(&mut stdout, "{}^{}", spaces, underscored)?;
    writeln!(&mut stdout, "{}{}\n", spaces, error_msg)?;
    no_color(&mut stdout)?;
    stdout.flush()?; // required to ensure that no_color is applied

    Ok(())
}

pub fn show_error_in_file(alias: &ModuleAlias, source: &String, error: Box<dyn CompileError>) {
    let (start, end) = error.get_position_window();
    let error_window = adjust_error_window(&source, start, end);
    let error_msg = error.get_message();

    show_error(&source, alias, error_window, error_msg).unwrap();
}
