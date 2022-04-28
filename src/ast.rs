use crate::types::ParsedType;

#[derive(Debug, PartialEq)]
pub struct FileAst {
    pub imports: Vec<ImportDecl>,
    pub functions: Vec<FunctionDecl>,
    pub types: Vec<ClassDecl>,
}

#[derive(Debug, PartialEq)]
pub struct ImportDecl {
    // Path to module, e.g. `from module.sub..` -> ["module", "sub"]
    pub module_path: Vec<String>,

    // NOTE: typenames is not Vec<Type> because only non-builtins are imported
    // so all of the imported types are Type::TypeIdentifier (this is checked by parser)
    pub typenames: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypedNamedObject {
    pub typename: ParsedType,
    pub name: String,
}

// TODO: think about removing these clone trains from object and function decl
// these structs are too big
#[derive(Debug, PartialEq)]
pub struct ClassDecl {
    pub is_active: bool,
    pub name: String,
    pub fields: Vec<TypedNamedObject>,
    pub methods: Vec<FunctionDecl>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDecl {
    pub rettype: Option<ParsedType>,
    pub name: String,
    pub args: Vec<TypedNamedObject>,
    pub statements: Vec<StatementWithPos>,
}

#[derive(Debug, PartialEq)]
pub struct StatementWithPos {
    pub statement: Statement,
    pub pos: usize,
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    IfElse {
        condition: ExprWithPos,
        if_body: Vec<StatementWithPos>,
        elif_bodies: Vec<(ExprWithPos, Vec<StatementWithPos>)>,
        else_body: Vec<StatementWithPos>,
    },
    While {
        condition: ExprWithPos,
        body: Vec<StatementWithPos>,
    },
    Foreach {
        item_name: String,
        iterable: ExprWithPos,
        body: Vec<StatementWithPos>,
    },
    Break,
    Continue,
    Return(Option<ExprWithPos>),
    Assign {
        left: ExprWithPos,
        right: ExprWithPos,
    },
    VarDecl(ParsedType, String),
    VarDeclWithAssign(ParsedType, String, ExprWithPos),
    SendMessage {
        active: ExprWithPos,
        method: String,
        args: Vec<ExprWithPos>,
    },
    // TODO: SWaitMessage
    Expr(ExprWithPos),
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    IsEqual,
    IsNotEqual,
    And,
    Or,
}

// priority from lowest to highest
// OR, AND
// IsEqual, IsEqual
// gt gte lt lte
// plus minus
// divide mult
// all unary ops
// grouped exprs

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOp {
    Not,
    Negate,
}

// TODO : exceptions lead to message being discarder + logs!!
// This means that if we do something like array[-1], we do not handle it, lol
// maybe save state of the actor before running it? (2x memory for this)

#[derive(Debug, PartialEq)]
pub struct ExprWithPos {
    pub expr: Expr,
    pub pos_first: usize,
    pub pos_last: usize,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    String(String),
    Bool(bool),
    Nil,
    Float(f64),

    This,
    Identifier(String),

    UnaryOp { op: UnaryOp, operand: Box<ExprWithPos> },
    BinOp { left: Box<ExprWithPos>, right: Box<ExprWithPos>, op: BinaryOp },

    ListAccess { list: Box<ExprWithPos>, index: Box<ExprWithPos> },
    ListValue(Vec<ExprWithPos>),
    TupleValue(Vec<ExprWithPos>),

    FunctionCall { function: String, args: Vec<ExprWithPos> },
    MethodCall { object: Box<ExprWithPos>, method: String, args: Vec<ExprWithPos> },
    FieldAccess { object: Box<ExprWithPos>, field: String },
    OwnMethodCall { method: String, args: Vec<ExprWithPos> },
    OwnFieldAccess { field: String },

    NewClassInstance { typename: String, args: Vec<ExprWithPos> },
    SpawnActive { typename: String, args: Vec<ExprWithPos> },
}
