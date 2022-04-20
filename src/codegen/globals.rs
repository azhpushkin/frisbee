use super::constants::Constant;

pub struct Globals {
    pub constants: Vec<Constant>,
}

impl Globals {
    pub fn new() -> Self {
        Globals { constants: vec![] }
    }

    pub fn get_constant(&mut self, constant: Constant) -> u8 {
        let existing = self.constants.iter().enumerate().find(|(_, c)| *c == &constant);
        match existing {
            Some((pos, _)) => pos as u8,
            None => {
                self.constants.push(constant);
                (self.constants.len() - 1) as u8
            }
        }
    }
}
