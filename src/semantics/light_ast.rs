use crate::ast::Type;

#[derive(Debug)]
pub enum LStatement {
    IfElse {
        condition: LExprTyped,
        ifbody: Vec<LStatement>,
        elsebody: Vec<LStatement>,
    },
    While {
        condition: LExprTyped,
        body: Vec<LStatement>,
    },
    Break,
    Continue,
    Return(LExprTyped),
    DeclareVar {
        var_type: Type,
        name: String,
    },
    AssignVar {
        name: String,
        value: LExprTyped,
    },
    Expression(LExprTyped),
    // TODO: send message
}

#[derive(Debug)]
pub enum RawOperator {
    UnaryNegateInt,
    AddInts,
    SubInts,
    MulInts,
    DivInts,
    GreaterInts,
    LessInts,
    EqualInts,

    UnaryNegateFloat,
    AddFloats,
    SubFloats,
    MulFloats,
    DivFloats,
    GreaterFloats,
    LessFloats,
    EqualFloats,

    UnaryNegateBool,
    // TODO: think about this a little more
}

#[derive(Debug)]
pub struct LExprTyped {
    pub expr: LExpr,
    pub expr_type: Type,
}

impl LExprTyped {
    pub fn int(value: i64) -> Self {
        LExprTyped { expr: LExpr::Int(value), expr_type: Type::Int }
    }
    pub fn float(value: f64) -> Self {
        LExprTyped { expr: LExpr::Float(value), expr_type: Type::Float }
    }
    pub fn bool(value: bool) -> Self {
        LExprTyped { expr: LExpr::Bool(value), expr_type: Type::Bool }
    }
    pub fn string(value: String) -> Self {
        LExprTyped { expr: LExpr::String(value), expr_type: Type::String }
    }
}

#[derive(Debug)]
pub enum LExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Nil,
    Float(f64),

    GetVar(String),

    ApplyOp { operator: RawOperator, operands: Vec<LExprTyped> },
    CallFunction { name: String, args: Vec<LExprTyped> },
    // TODO: all others

    // TODO: spawn!
}
