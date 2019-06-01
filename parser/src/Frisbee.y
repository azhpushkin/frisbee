{
module Frisbee where
import Tokens
}


%name frisbee
%tokentype { Token }
%error { parseError }
%token
  "active"                { TActive _ }
  "passive"                { TPassive _ }
  "new"                    { TNew _ }
  "spawn"                    { TSpawn _ }
  "import"              { TImport _ }
  "from"              { TFrom _ }
  typeident                                { TTypeIdent _ $$ }
  
  "Void"                { TVoid _ }
  "def"                { TDef _ }
  "return"                              { TReturn _ }
  
  "val"                { TVal _ }
  "String"                { TString _ }
  "Int"                        { TInt _ }
  "Bool"                { TBool _ }
  "?"         { TMaybe _ }
  "["                    { TLeftBrack _ }
  "]"                    { TRightBrack _ }


  "io"                        { TIo _ }
  "if"                        { TIf _ }
  "else"                { TElse _ }
  "void"                { TVoidValue _ }
  "true"                { TTrue _ }
  "false"                { TFalse _ }
  "this"                { TThis _ }
  "while"                { TWhile _ }
  integer_literal            { TIntLiteral _ $$ }
  string_literal            { TStringLiteral _ $$ }
  ident                                { TIdent _ $$ }
  "{"                         { TLeftBrace _ }
  "}"                    { TRightBrace _ }
  ","                    { TComma _ }
  
  op                                    { TOp _ $$}
  comop                                 { TComOp _ $$ }
  "("                                   { TLeftParen _ }
  ")"                                   { TRightParen _ }
  ";"                                   { TSemiColon _ }
  "."                                   { TPeriod _ }
  
  "not"                                   { TNot _ }
  "="                                   { TEquals _ }
  "<="                                   { TWaitMessage _ }
  "!"                                   { TSendMessage _ }

%left op
%nonassoc comop
%%

Program : 
        ImportDeclList ObjectDeclList { Program $1 $2 }


ImportDeclList :
        "from" ident "import" ImportIdentList ";" ImportDeclList { ImportDeclList $2 $4 $6 }
        |                                                  { ImportDeclListEmpty }


ImportIdentList :
        typeident "," ImportIdentList { ImportIdentList $1 $3 }
        | typeident               { ImportIdentList $1 ImportIdentListEmpty }


ObjectDeclList :
          ObjectDecl     { ObjectDeclList $1 OEmpty }
          | ObjectDecl ObjectDeclList { ObjectDeclList $1 $2 }
          |             { OEmpty }

ObjectDecl : 
            "active" typeident "{" VarDeclList MethodDeclList "}"    { ActiveDecl  $2 $4 $5 }
          | "passive" typeident "{" VarDeclList MethodDeclList "}"   { PassiveDecl $2 $4 $5 }


MethodDeclList :
     MethodDecl                   { MethodDeclList $1 MEmpty }
     | MethodDecl MethodDeclList  { MethodDeclList $1 $2 }
     |                            { MEmpty }

MethodDecl : 
       "def" Type ident "(" FormalList ")" "{" StatementList "}" { MethodDecl $2 $3 $5 $8 }

VarDeclList :
     Type ident ";" { VarDeclList $1 $2 VEmpty }
     | Type ident ";" VarDeclList { VarDeclList $1 $2 $4 }
     |              { VEmpty }

FormalList :
     Type ident       { FormalList $1 $2 FEmpty }
     | Type ident "," FormalList { FormalList $1 $2 $4 }
     |                  { FEmpty }

Type :
     "val"       { TypeAnonymous }
     | Type "?"    { TypeMaybe $1 }
     | "[" Type "]"    { TypeArray $2 }
     | "Void"       { TypeVoid }
     | "Int"    { TypeInt }
     | "String"    { TypeString }
     | "Bool"    { TypeBool }
     | typeident    { TypeIdent $1 }
    

Statement :
    "{" StatementList "}"                            { SList $2 }
    | "if" "(" Exp ")" Statement "else" Statement  { SIfElse $3 $5 $7 }
    | "if" "(" Exp ")" Statement                  { SIfElse $3 $5 (SList Empty) }
    | "while" "(" Exp ")" Statement                { SWhile $3 $5 }
    | "return" Exp ";"                              { SReturn $2 }
    | ident "=" Exp ";"                              { SEqual $1 $3 }
    | Type ident ";"                              { SVarDecl $1 $2 }
    | Type ident "=" Exp ";"                              { SVarDeclEqual $1 $2 $4 }
    | Exp "." ident   "=" Exp ";"                              { SEqualField $1 $3 $5 }
    | Exp "!" ident "(" ExpList ")" ";"   { SSendMessage $1 $3 $5}
    | ident "<=" Exp "!" ident "(" ExpList ")" ";"   { SWaitMessage $1 $3 $5 $7 }
    | ident "[" Exp "]" "=" Exp ";"                  { SArrayEqual $1 $3 $6 }
    | Exp   ";"                    { SExp $1}

StatementList :
    Statement               { StatementList Empty $1 }
    | StatementList Statement   { StatementList $1 $2 }

Exp : 
    Exp op Exp                        { ExpOp $1 $2 $3}
    | Exp comop Exp                   { ExpComOp $1 $2 $3}
    | Exp "[" Exp "]"                 { ExpArrayGet $1 $3}
    | Exp "." ident "(" ExpList ")"   { ExpFCall $1 $3 $5}
    | Exp "." ident                   { ExpFieldAccess $1 $3}
    | integer_literal                 { ExpInt $1}
    | string_literal                 { ExpString $1}
    | "void"                          { ExpVoid }
    | "true"                          { ExpBool True}
    | "false"                         { ExpBool False}
    | ident                           { ExpIdent $1}
    | "this"                          { ExpThis }
    | "io"                          { ExpIO }
    | "new" typeident "(" ExpList")"             { ExpNewPassive $2 $4}
    | "spawn" typeident "(" ExpList ")"             { ExpSpawnActive $2 $4}
    | "not" Exp                         { ExpNot $2}
    | "(" Exp ")"                     { ExpExp $2}

ExpList :
        Exp "," ExpList  { ExpList $1 $3 }  
        | Exp               { ExpList $1 ExpListEmpty }
        |                 { ExpListEmpty }


{
parseError :: [Token] -> a
parseError tokenList =
  let pos = tokenPosn $ head tokenList
  in error ("parse error at line " ++ show(getLineNum(pos)) ++ " and column " ++ show(getColumnNum(pos)))



-- PYTHON START HERE

data Program = Program ImportDeclList ObjectDeclList  -- imports, objects
      deriving (Show, Eq)


data ImportDeclList
    = ImportDeclList String ImportIdentList ImportDeclList  -- module, typenames, tail
    | ImportDeclListEmpty  -- 
    deriving (Show, Eq)


data ObjectDeclList
    = ObjectDeclList ObjectDecl ObjectDeclList  -- head, tail
    | OEmpty -- 
  deriving (Show, Eq)

data ObjectDecl
    = ActiveDecl  String VarDeclList MethodDeclList  -- name, vars, methods
    | PassiveDecl String VarDeclList MethodDeclList  -- name, vars, methods
  deriving (Show, Eq)


data MethodDeclList
    = MethodDeclList MethodDecl MethodDeclList  -- head, tail
    | MEmpty  -- 
    deriving (Show, Eq)


data MethodDecl
    = MethodDecl Type String FormalList StatementList  -- type, name, args, statements
    deriving (Show, Eq)

data VarDeclList =
    VarDeclList Type String VarDeclList  -- typename, name, tail
    | VEmpty  --
    deriving (Show, Eq)

data FormalList = 
    FormalList Type String FormalList  -- typename, name, tail
    | FEmpty  --
  deriving (Show, Eq)

data Type =
      TypeAnonymous  -- 
    | TypeMaybe Type  -- type
    | TypeArray Type  -- type
    | TypeInt  --
    | TypeVoid  --
    | TypeBool  --
    | TypeString  --
    | TypeIdent String  -- name
    deriving (Show, Eq)

data Statement
    = SList StatementList  -- statements
    | SIfElse Exp Statement Statement  -- condition, if_body, else_body
    | SWhile Exp Statement  -- condition, body
    | SReturn Exp  -- expr
    | SEqual String Exp  -- name, expr
    | SVarDeclEqual Type String Exp  -- type, name, expr
    | SVarDecl Type String  -- type, name
    | SEqualField Exp String Exp  -- object, field, expr
    | SArrayEqual String Exp Exp  -- name, index, expr
    | SSendMessage Exp String ExpList  -- object, method, args
    | SWaitMessage String Exp String ExpList  -- result_name, object, method, args
    | SExp Exp  -- expr
    deriving (Show, Eq)

data StatementList
    = StatementList StatementList Statement -- tail, head
    | Empty -- 
    deriving (Show, Eq)


data Exp
    = ExpOp Exp String Exp  -- left, operator, right
    | ExpComOp Exp String Exp  -- left, operator, right
    | ExpArrayGet Exp Exp -- array, index
    | ExpFCall Exp String ExpList  -- object, method, args
    | ExpFieldAccess Exp String  -- object, field
    | ExpInt Int  -- value
    | ExpString String  -- value
    | ExpBool Bool -- value
    | ExpVoid -- value
    | ExpIdent String -- name
    | ExpNewPassive String ExpList  -- typename, args 
    | ExpSpawnActive String ExpList  -- typename, args
    | ExpExp Exp -- expr
    | ExpThis  -- 
    | ExpIO  -- 
    | ExpNot Exp  -- operand
    deriving (Show, Eq)

data ExpList
    = ExpList Exp ExpList  -- head, tail
    | ExpListEmpty    -- 
    deriving (Show, Eq)

data ImportIdentList 
    = ImportIdentList String ImportIdentList  -- typename, tail 
    | ImportIdentListEmpty   -- 
    deriving (Show, Eq)

-- PYTHON END HERE


}