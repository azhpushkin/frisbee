struct Program {
    imports: Vec<ImportDecl>,
    passive: Vec<ObjectDecl>,
    active: Vec<ObjectDecl>
}

struct ImportDecl {
    module: String,
    typenames: Vec<String> // not Type because only non-builtins are imported
}

struct TypedNamedObject {
    typename: Type,
    name: String
}

struct ObjectDecl {
    is_active: bool,
    name: String,
    fields: Vec<TypedNamedObject>,
    methods: Vec<MethodDecl>,
}

struct MethodDecl {
    rettype: Type,
    name: String,
    args: Vec<TypedNamedObject>,
    statements: Vec<Statement>,
}

enum Type {
    // TODO: TypeAnonymous
    // TODO: TypeMaybe (Type),
    TypeArray (Box<Type>),
    TypeInt,
    TypeFloat,
    TypeNil,
    TypeBool,
    TypeString,
    TypeIdent (String),
}

enum Statement {
    SIfElse { condition: Expr, ifbody: Vec<Statement>, elsebody: Vec<Statement>},
    SWhile {condition: Expr, body: Vec<Statement>},
    SReturn(Expr),
    SEqual {left: Expr, right: Expr},
    SVarDeclEqual(Type, String, Expr),
    SSendMessage { active: Expr, method: String, args: Vec<Expr>},
    // TODO: SWaitMessage
    SExpr(Expr),
}

enum Expr {
    ExprUnaryOp {op: String, operand: Box<Expr>},
    ExprBinOp {left: Box<Expr>, right: Box<Expr>, op: String},
    ExprArrayAccess {array: Box<Expr>, index: Box<Expr>},
    ExprArrayValue (Vec<Box<Expr>>),
    ExprFuncCall {object: Box<Expr>, method: String, args: Vec<Expr>},
    ExpFieldAccess {object: Box<Expr>, field: String},
    ExprInt(i32),
    ExprString(String),
    ExprBool(bool),
    ExprNil,
    ExprFloat(f32),
    ExprIdentifier(String),
    ExprNewPassive {typename: String, args: Vec<Expr>},
    ExprSpawnActive {typename: String, args: Vec<Expr>},
    ExprThis,
    ExprCaller,
}

