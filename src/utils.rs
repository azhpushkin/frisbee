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
