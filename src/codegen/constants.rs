use crate::vm::op;

#[derive(Debug)]
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
}

pub fn constants_to_bytecode(data: &Vec<Constant>) -> Vec<u8> {
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
