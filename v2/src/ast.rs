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
    TypeArray(Box<Type>),
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
pub enum BinaryOperator {
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

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    ExprUnaryOp { op: UnaryOperator, operand: Box<Expr> },
    ExprBinOp { left: Box<Expr>, right: Box<Expr>, op: BinaryOperator },
    ExprArrayAccess { array: Box<Expr>, index: Box<Expr> },
    ExprArrayValue(Vec<Box<Expr>>),
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
    ExprCaller,
}
