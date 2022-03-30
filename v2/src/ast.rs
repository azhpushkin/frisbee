#[derive(Debug, PartialEq)]
pub struct FileAst {
    pub imports: Vec<ImportDecl>,
    pub functions: Vec<FunctionDecl>,
    pub types: Vec<ObjectDecl>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ModulePath(pub Vec<String>);

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct ModulePathAlias(String);

impl Into<ModulePathAlias> for ModulePath {
    fn into(self) -> ModulePathAlias {
        ModulePathAlias(self.0.join("."))
    }
}
impl ModulePath {
    pub fn alias(&self) -> ModulePathAlias {
        self.clone().into()
    }
    pub fn get_vec(&self) -> Vec<String> {
        self.0.clone()
    } // TODO: lifetime here to avoid copy?
}
impl ModulePathAlias {
    pub fn as_str(&self) -> String {
        self.0.clone()
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
pub struct ObjectDecl {
    pub is_active: bool,
    pub name: String,
    pub fields: Vec<TypedNamedObject>,
    pub methods: Vec<FunctionDecl>,
}

#[derive(Debug, PartialEq)]
pub struct FunctionDecl {
    pub rettype: Type,
    pub name: String,
    pub args: Vec<TypedNamedObject>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
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
    SForeach {
        itemname: String,
        iterable: Expr,
        body: Vec<Statement>,
    },
    SBreak,
    SContinue,
    SReturn(Expr),
    SAssign {
        left: Expr,
        right: Expr,
    },
    SVarDecl(Type, String),
    SVarDeclEqual(Type, String, Expr),
    SSendMessage {
        active: Expr,
        method: String,
        args: Vec<Expr>,
    },
    // TODO: SWaitMessage
    // TODO: break and continue?
    SExpr(Expr),
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
    ExprUnaryOp { op: UnaryOp, operand: Box<Expr> },
    ExprBinOp { left: Box<Expr>, right: Box<Expr>, op: BinaryOp },
    ExprListAccess { list: Box<Expr>, index: Box<Expr> },
    ExprListValue(Vec<Expr>),
    ExprTupleValue(Vec<Expr>),
    ExprFunctionCall { function: String, args: Vec<Expr> },
    ExprMethodCall { object: Box<Expr>, method: String, args: Vec<Expr> },
    ExprFieldAccess { object: Box<Expr>, field: String },
    ExprOwnMethodCall { method: String, args: Vec<Expr> },
    ExprOwnFieldAccess { field: String },
    ExprInt(i32),
    ExprString(String),
    ExprBool(bool),
    ExprNil,
    ExprFloat(f32),
    ExprIdentifier(String),
    ExprNewClassInstance { typename: String, args: Vec<Expr> },
    ExprSpawnActive { typename: String, args: Vec<Expr> },
    ExprThis,
}
