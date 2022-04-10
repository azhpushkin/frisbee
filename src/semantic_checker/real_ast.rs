use super::real_type::RType;



pub enum RStatement {
    IfElse {condition: RExpr, ifbody: Vec<RStatement>, elsebody: Vec<RStatement>},
    While {condition: RExpr, body: Vec<RStatement>},
    Break,
    Continue,
    Return(RExpr),
    DeclareVar{ttype: RType, name: String},
    AssignVar{name: String, value: RExpr},
    Expression(RExpr),
    // TODO: send message
}

// #[rustfmt::skip]
pub enum Operator {
    UnaryNegateInt,
    AddInts,
    SubInts,
    MulInts,
    DivInts,

    UnaryNegateFloat,
    AddFloats,
    SubFloats,
    MulFloats,
    DivFloats,

    UnaryNegateBool,
    // TODO: think about this a little more
}

pub enum RExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Nil,
    Float(f64),

    GetVar(String),

    ApplyOp {op: Operator, operands: Vec<RExpr>},
    CallFunction {name: String, args: Vec<RExpr>},

    // TODO: all others
    
    // TODO: spawn!
}