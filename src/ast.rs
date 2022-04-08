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
    pub rettype: Type,
    pub name: String,
    pub args: Vec<TypedNamedObject>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    // Primitive types
    TypeInt,
    TypeFloat,
    TypeNil,
    TypeBool,
    TypeString,

    // Type wrappers
    TypeList(Box<Type>),
    TypeTuple(Vec<Type>),
    TypeMaybe(Box<Type>),

    // User-defined type
    TypeIdent(String),

    // Used for empty arrays, nil values and future `let` expression
    TypeAnonymous,

    TypeIdentQualified(ModulePathAlias, String),
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    SIfElse { condition: ExprRaw, ifbody: Vec<Statement>, elsebody: Vec<Statement> },
    SWhile { condition: ExprRaw, body: Vec<Statement> },
    SForeach { itemname: String, iterable: ExprRaw, body: Vec<Statement> },
    SBreak,
    SContinue,
    SReturn(ExprRaw),
    SAssign { left: ExprRaw, right: ExprRaw },
    SVarDecl(Type, String),
    SVarDeclWithAssign(Type, String, ExprRaw),
    SSendMessage { active: ExprRaw, method: String, args: Vec<ExprRaw> },
    // TODO: SWaitMessage
    SExpr(ExprRaw),
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

pub trait ExprWithType {
    fn get_raw(&self) -> &ExprRaw;
    fn get_type(&self) -> &Type;
}

#[derive(Debug, PartialEq)]
pub enum ExprRaw {
    UnaryOp { op: UnaryOp, operand: Box<ExprRaw> },
    BinOp { left: Box<ExprRaw>, right: Box<ExprRaw>, op: BinaryOp },
    ListAccess { list: Box<ExprRaw>, index: Box<ExprRaw> },
    ListValue(Vec<ExprRaw>),
    TupleValue(Vec<ExprRaw>),
    FunctionCall { function: String, args: Vec<ExprRaw> },
    MethodCall { object: Box<ExprRaw>, method: String, args: Vec<ExprRaw> },
    FieldAccess { object: Box<ExprRaw>, field: String },
    OwnMethodCall { method: String, args: Vec<ExprRaw> },
    OwnFieldAccess { field: String },
    Int(i32),
    String(String),
    Bool(bool),
    Nil,
    Float(f32),
    Identifier(String),
    NewClassInstance { typename: String, args: Vec<ExprRaw> },
    SpawnActive { typename: String, args: Vec<ExprRaw> },
    This,
}

