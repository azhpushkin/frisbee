use crate::runtime::opcodes::op;

#[derive(Debug, PartialEq)]
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
}

fn constants_to_bytecode(data: &[Constant]) -> Vec<u8> {
    let mut res = vec![];
    for constant in data.iter() {
        match constant {
            Constant::Int(i) => {
                res.push(op::CONST_INT_FLAG);
                res.extend(i.to_be_bytes());
            }
            Constant::Float(f) => {
                res.push(op::CONST_FLOAT_FLAG);
                res.extend(f.to_be_bytes());
            }
            Constant::String(s) => {
                res.push(op::CONST_STRING_FLAG);
                res.extend((s.len() as u16).to_be_bytes());
                res.extend(s.as_bytes());
            }
        }
    }
    res.push(op::CONST_END_FLAG);

    res
}

pub struct ConstantsTable {
    table: Vec<Constant>,
}

impl ConstantsTable {
    pub fn new() -> Self {
        ConstantsTable { table: vec![] }
    }

    pub fn get_constant(&mut self, constant: Constant) -> u8 {
        let existing = self.table.iter().enumerate().find(|(_, c)| *c == &constant);
        match existing {
            Some((pos, _)) => pos as u8,
            None => {
                self.table.push(constant);
                (self.table.len() - 1) as u8
            }
        }
    }

    pub fn generate_bytecode(&self) -> Vec<u8> {
        constants_to_bytecode(&self.table)
    }
}
