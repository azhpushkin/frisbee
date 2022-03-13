#[derive(Debug, PartialEq)]
pub struct Program {
    pub imports: Vec<ImportDecl>,
    pub passive: Vec<ObjectDecl>,
    pub active: Vec<ObjectDecl>,
}

#[derive(Debug, PartialEq)]
pub struct ImportDecl {
    pub module: String,
    pub typenames: Vec<String>, // not Type because only non-builtins are imported
}

#[derive(Debug, PartialEq)]
pub struct TypedNamedObject {
    pub typename: Type,
    pub name: String,
}

#[derive(Debug, PartialEq)]
pub struct ObjectDecl {
    pub is_active: bool,
    pub name: String,
    pub fields: Vec<TypedNamedObject>,
    pub methods: Vec<MethodDecl>,
}

#[derive(Debug, PartialEq)]
pub struct MethodDecl {
    pub rettype: Type,
    pub name: String,
    pub args: Vec<TypedNamedObject>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
pub enum Type {
    // TODO: TypeAnonymous
    // TODO: TypeMaybe (Type),
    TypeList(Box<Type>),
    TypeTuple(Vec<Type>),
    TypeMaybe(Box<Type>),
    TypeInt,
    TypeFloat,
    TypeNil,
    TypeBool,
    TypeString,
    TypeIdent(String),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    SIfElse {
        condition: Expr,
        ifbody: Vec<Statement>,
        elsebody: Vec<Statement>,
    },
    SWhile {
        condition: Expr,
        body: Vec<Statement>,
    },
    SReturn(Expr),
    SEqual {
        left: Expr,
        right: Expr,
    },
    SVarDeclEqual(Type, String, Expr),
    SSendMessage {
        active: Expr,
        method: String,
        args: Vec<Expr>,
    },
    // TODO: SWaitMessage
    SExpr(Expr),
}

#[derive(Debug, PartialEq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Greater,
    GreaterThan,
    Less,
    LessThan,
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

// TODO : tuple vs 

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Not,
    Negate,
}

// TODO : exceptions lead to message being discarder + logs!!
// This means that if we do something like array[-1], we do not handle it, lol
// maybe save state of the actor before running it? (2x memory for this)
#[derive(Debug, PartialEq)]
pub enum Expr {
    ExprUnaryOp { op: UnaryOp, operand: Box<Expr> },
    ExprBinOp { left: Box<Expr>, right: Box<Expr>, op: BinaryOp },
    ExprListAccess { list: Box<Expr>, index: Box<Expr> },
    ExprListValue(Vec<Box<Expr>>),
    ExprFuncCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    ExpFieldAccess { object: Box<Expr>, field: String },
    ExprInt(i32),
    ExprString(String),
    ExprBool(bool),
    ExprNil,
    ExprFloat(f32),
    ExprIdentifier(String),
    ExprNewPassive { typename: String, args: Vec<Expr> },
    ExprSpawnActive { typename: String, args: Vec<Expr> },
    ExprThis,
    // ExprCaller,
}
