macro_rules! extract_result_if_ok {
    ($parse_result:expr) => {
        match $parse_result {
            Ok(res) => res,
            // Re-wrap pf parsing error is required to coerce type
            // from Result<T, ParseError> to Result<Program, ParseError>
            Err(t) => return Err(t),
        }
    };
}
pub(crate) use extract_result_if_ok;

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
