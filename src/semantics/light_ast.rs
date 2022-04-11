use super::real_type::RType;

#[derive(Debug)]
pub enum LStatement {
    IfElse { condition: LExpr, ifbody: Vec<LStatement>, elsebody: Vec<LStatement> },
    While { condition: LExpr, body: Vec<LStatement> },
    Break,
    Continue,
    Return(LExpr),
    DeclareVar { rtype: RType, name: String },
    AssignVar { name: String, value: LExpr },
    Expression(LExpr),
    // TODO: send message
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum LExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Nil,
    Float(f64),

    GetVar(String),

    ApplyOp { op: Operator, operands: Vec<LExpr> },
    CallFunction { name: String, args: Vec<LExpr> },
    // TODO: all others

    // TODO: spawn!
}
