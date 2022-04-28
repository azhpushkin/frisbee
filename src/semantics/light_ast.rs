use crate::types::VerifiedType;

use crate::symbols::{SymbolFunc, SymbolType};

#[derive(Debug)]
pub enum LStatement {
    IfElse {
        condition: LExprTyped,
        if_body: Vec<LStatement>,
        else_body: Vec<LStatement>,
    },
    While {
        condition: LExprTyped,
        body: Vec<LStatement>,
    },
    Break,
    Continue,
    Return(LExprTyped),
    DeclareVar {
        var_type: VerifiedType,
        name: String,
    },
    DeclareAndAssignVar {
        var_type: VerifiedType,
        name: String,
        value: LExprTyped,
    },
    // TODO: change to generic assign
    // assign to name, field, tuple or list only allowed
    AssignLocal {
        name: String,
        // [1, 2] means <var>[1][2]
        // these indexes are verified and flattened for simplicity
        tuple_indexes: Vec<usize>,
        value: LExprTyped,
    },
    AssignToField {
        object: LExprTyped, // box to avoid hustle of unboxing from LExpr::AccessField
        field: String,
        tuple_indexes: Vec<usize>,
        value: LExprTyped,
    },
    AssignToList {
        list: LExprTyped,
        index: LExprTyped,
        tuple_indexes: Vec<usize>,
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
    EqualBools,
    AndBools,
    OrBools,

    EqualStrings,
    AddStrings,
    // TODO: think about this a little more
}

#[derive(Debug)]
pub struct LExprTyped {
    pub expr: LExpr,
    pub expr_type: VerifiedType,
}

impl LExprTyped {
    pub fn int(value: i64) -> Self {
        LExprTyped { expr: LExpr::Int(value), expr_type: VerifiedType::Int }
    }
    pub fn float(value: f64) -> Self {
        LExprTyped { expr: LExpr::Float(value), expr_type: VerifiedType::Float }
    }
    pub fn bool(value: bool) -> Self {
        LExprTyped { expr: LExpr::Bool(value), expr_type: VerifiedType::Bool }
    }
    pub fn string(value: String) -> Self {
        LExprTyped { expr: LExpr::String(value), expr_type: VerifiedType::String }
    }
}

#[derive(Debug)]
pub enum LExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Float(f64),

    GetVar(String),
    AccessTupleItem { tuple: Box<LExprTyped>, index: usize },

    TupleValue(Vec<LExprTyped>),
    ListValue { item_type: VerifiedType, items: Vec<LExprTyped> },

    ApplyOp { operator: RawOperator, operands: Vec<LExprTyped> },
    CallFunction { name: SymbolFunc, return_type: VerifiedType, args: Vec<LExprTyped> },

    AccessField { object: Box<LExprTyped>, field: String },
    AccessListItem { list: Box<LExprTyped>, index: Box<LExprTyped> },

    Allocate { typename: SymbolType },
    // TODO: spawn!
}
