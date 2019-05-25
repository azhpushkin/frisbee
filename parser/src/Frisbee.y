{
module Frisbee where
import Tokens
}


%name frisbee
%tokentype { Token }
%error { parseError }
%token
  "active"				{ TActive _ }
  "passive"				{ TPassive _ }
  "new"					{ TNew _ }
  "spawn"					{ TSpawn _ }
  typeident		                        { TTypeIdent _ $$ }
  
  "Void"				{ TVoid _ }
  "def"				{ TDef _ }
  "return"                              { TReturn _ }
  
  "val"				{ TVal _ }
  "String"				{ TString _ }
  "Int"				        { TInt _ }
  "Bool"				{ TBool _ }
  "?"         { TMaybe _ }
  "["					{ TLeftBrack _ }
  "]"					{ TRightBrack _ }


  "if"				        { TIf _ }
  "else"				{ TElse _ }
  "true"				{ TTrue _ }
  "false"				{ TFalse _ }
  "this"				{ TThis _ }
  "while"				{ TWhile _ }
  integer_literal			{ TIntLiteral _ $$ }
  string_literal			{ TStringLiteral _ $$ }
  ident		                        { TIdent _ $$ }
  "{"	 	 	   		{ TLeftBrace _ }
  "}"					{ TRightBrace _ }
  ","					{ TComma _ }
  
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

%nonassoc op
%nonassoc comop
%%

Program : 
        ObjectDeclList { Program $1 }

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
       "def" Type ident "(" FormalList ")" "{" VarDeclList StatementList "}" { MethodDecl $2 $3 $5 $8 $9 }

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
    | "true"                          { ExpBool True}
    | "false"                         { ExpBool False}
    | ident                           { ExpIdent $1}
    | "this"                          { ExpThis }
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


data Program = Program ObjectDeclList
      deriving (Show, Eq)


data ObjectDeclList
    = ObjectDeclList ObjectDecl ObjectDeclList
    | OEmpty
  deriving (Show, Eq)

data ObjectDecl
    = ActiveDecl  Ident VarDeclList MethodDeclList
    | PassiveDecl Ident VarDeclList MethodDeclList
  deriving (Show, Eq)


data MethodDeclList
    = MethodDeclList MethodDecl MethodDeclList
    | MEmpty
    deriving (Show, Eq)
data MethodDecl
    = MethodDecl Type Ident FormalList VarDeclList StatementList
    deriving (Show, Eq)

data VarDeclList =
    VarDeclList Type Ident VarDeclList
    | VEmpty
    deriving (Show, Eq)

data FormalList = 
    FormalList Type Ident FormalList
    | FEmpty
  deriving (Show, Eq)

data Type =
      TypeAnonymous
    | TypeMaybe Type
    | TypeArray Type
    | TypeInt
    | TypeVoid
    | TypeBool
    | TypeString
    | TypeIdent Ident
    deriving (Show, Eq)

data Statement
    = Statement String
    | SList StatementList
    | SIfElse Exp Statement Statement
    | SWhile Exp Statement
    | SReturn Exp
    | SEqual Ident Exp
    | SEqualField Exp Ident Exp
    | SArrayEqual Ident Exp Exp
    | StatementError
    | SSendMessage Exp Ident ExpList
    | SWaitMessage Ident Exp Ident ExpList
    | SExp Exp
    deriving (Show, Eq)

data StatementList
    = StatementList StatementList Statement 
    | Empty
    deriving (Show, Eq)


data Exp
    = Exp String
    | ExpOp Exp String Exp
    | ExpComOp Exp String Exp
    | ExpArrayGet Exp Exp -- "Exp [ Exp ]"
    | ExpFCall Exp Ident ExpList -- Exp . Ident ( ExpList )
    | ExpFieldAccess Exp Ident
    | ExpInt Int
    | ExpString String
    | ExpBool Bool -- True or False
    | ExpIdent Ident
    | ExpNewPassive Ident ExpList -- new Ident ()
    | ExpSpawnActive Ident ExpList -- new Ident ()
    | ExpExp Exp -- Exp ( Exp )
    | ExpThis
    | ExpNot Exp
    | ExpError
    deriving (Show, Eq)

type Ident = String
type Integer_Literal = Int
data ExpList
    = ExpList Exp ExpList
    | ExpListEmpty    
    deriving (Show, Eq)


}