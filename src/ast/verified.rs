use std::collections::HashMap;

use crate::alias::ModuleAlias;
use crate::symbols::{SymbolFunc, SymbolType};
use crate::types::VerifiedType;

#[derive(Debug)]
pub struct CustomType {
    pub name: SymbolType,
    pub is_active: bool,
    pub fields: TypedFields,
}

#[derive(Debug)]
pub struct RawFunction {
    pub name: SymbolFunc,
    pub return_type: VerifiedType,
    pub args: TypedFields,
    pub body: Vec<VStatement>,
    pub locals: Vec<(String, VerifiedType)>,

    pub short_name: String,
    pub method_of: Option<SymbolType>,
    pub is_constructor: bool,
    pub defined_at: ModuleAlias,
}

/// Simple ordered HashMap for typed and ordered fields
/// (used by function arguments and class types)
#[derive(Debug)]
pub struct TypedFields {
    // TODO: remove pub, add methods for iter(), len() and add_this
    pub types: Vec<VerifiedType>,
    pub names: HashMap<usize, String>,
}

#[derive(Debug)]
pub enum VStatement {
    IfElse {
        condition: VExprTyped,
        if_body: Vec<VStatement>,
        else_body: Vec<VStatement>,
    },
    While {
        condition: VExprTyped,
        body: Vec<VStatement>,
    },
    Break,
    Continue,
    Return(VExprTyped),
    // Assign to local variable on stack with compile-time calculated offset
    AssignLocal {
        name: String,
        // [1, 2] means <var>[1][2]
        // these indexes are verified and flattened for simplicity
        tuple_indexes: Vec<usize>,
        value: VExprTyped,
    },
    // Assign to heap variable with compile-time calculated offset
    AssignToField {
        object: VExprTyped, // box to avoid hustle of unboxing from LExpr::AccessField
        field: String,
        tuple_indexes: Vec<usize>,
        value: VExprTyped,
    },
    // Assign to heap variable with runtime-calculated offset
    AssignToList {
        list: VExprTyped,
        index: VExprTyped,
        tuple_indexes: Vec<usize>,
        value: VExprTyped,
    },

    Expression(VExprTyped),
}

#[derive(Debug, PartialEq)]
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
pub struct VExprTyped {
    pub expr: VExpr,
    pub expr_type: VerifiedType,
}

// DO NOT ADD CLONE as cloning an expression might
#[derive(Debug)]
pub enum VExpr {
    Int(i64),
    String(String),
    Bool(bool),
    Float(f64),

    Dummy(VerifiedType), // used for Maybe types
    CompareMaybe {
        left: Box<VExprTyped>,
        right: Box<VExprTyped>,
        eq_op: RawOperator,
    },

    GetVar(String),
    AccessTupleItem {
        tuple: Box<VExprTyped>,
        index: usize,
    },

    TupleValue(Vec<VExprTyped>),
    ListValue {
        item_type: VerifiedType,
        items: Vec<VExprTyped>,
    },

    ApplyOp {
        operator: RawOperator,
        operands: Vec<VExprTyped>,
    },
    TernaryOp {
        condition: Box<VExprTyped>,
        if_true: Box<VExprTyped>,
        if_false: Box<VExprTyped>,
    },
    CallFunction {
        name: SymbolFunc,
        return_type: VerifiedType,
        args: Vec<VExprTyped>,
    },

    AccessField {
        object: Box<VExprTyped>,
        field: String,
    },
    AccessListItem {
        list: Box<VExprTyped>,
        index: Box<VExprTyped>,
    },

    Allocate {
        typename: SymbolType,
    },
    // TODO: spawn!
}

impl TypedFields {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &VerifiedType)> {
        self.types.iter().enumerate().map(move |(i, t)| (&self.names[&i], t))
    }
}
