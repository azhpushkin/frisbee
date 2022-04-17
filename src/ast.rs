#[derive(Debug, PartialEq)]
pub struct FileAst {
    pub imports: Vec<ImportDecl>,
    pub functions: Vec<FunctionDecl>,
    pub types: Vec<ClassDecl>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModulePath(pub Vec<String>);

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ModulePathAlias(pub String);

impl ModulePath {
    pub fn alias(&self) -> ModulePathAlias {
        ModulePathAlias(self.alias_str())
    }
    pub fn alias_str(&self) -> String {
        self.0.join(".")
    }
}

#[derive(Debug, PartialEq)]
pub struct ImportDecl {
    // Path to module, e.g. `from module.sub..` -> ["module", "sub"]
    pub module_path: ModulePath,

    // NOTE: typenames is not Vec<Type> because only non-builtins are imported
    // so all of the imported types are Type::TypeIdentifier (this is checked by parser)
    pub typenames: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypedNamedObject {
    pub typename: Type,
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
    pub rettype: Option<Type>,
    pub name: String,
    pub args: Vec<TypedNamedObject>,
    pub statements: Vec<Statement>,
}

pub type Type = crate::types::Type;

#[derive(Debug, PartialEq)]
pub enum Statement {
    IfElse {
        condition: Expr,
        if_body: Vec<Statement>,
        elif_bodies: Vec<(Expr, Vec<Statement>)>,
        else_body: Vec<Statement>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    Foreach {
        itemname: String,
        iterable: Expr,
        body: Vec<Statement>,
    },
    Break,
    Continue,
    Return(Option<Expr>),
    Assign {
        left: Expr,
        right: Expr,
    },
    VarDecl(Type, String),
    VarDeclWithAssign(Type, String, Expr),
    SendMessage {
        active: Expr,
        method: String,
        args: Vec<Expr>,
    },
    // TODO: SWaitMessage
    Expr(Expr),
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
pub enum Expr {
    Int(i64),
    String(String),
    Bool(bool),
    Nil,
    Float(f64),

    This,
    Identifier(String),

    UnaryOp { op: UnaryOp, operand: Box<Expr> },
    BinOp { left: Box<Expr>, right: Box<Expr>, op: BinaryOp },

    ListAccess { list: Box<Expr>, index: Box<Expr> },
    ListValue(Vec<Expr>),
    TupleValue(Vec<Expr>),

    FunctionCall { function: String, args: Vec<Expr> },
    MethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    FieldAccess { object: Box<Expr>, field: String },
    OwnMethodCall { method: String, args: Vec<Expr> },
    OwnFieldAccess { field: String },

    NewClassInstance { typename: String, args: Vec<Expr> },
    SpawnActive { typename: String, args: Vec<Expr> },
}
