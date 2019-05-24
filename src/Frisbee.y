{
module Frisbee where
import Tokens
}


%name newl
%tokentype { Token }
%error { parseError }
%token
  "class"				{ TClass _ }
  "new"					{ TNew _ }
"String"				{ TString _ }
  "void"				{ TVoid _ }
  "main"				{ TMain _ }
  "public"				{ TPublic _ }
  "return"                              { TReturn _ }
  "extends"				{ TExtend _ }
  "int"				        { TInt _ }
  "boolean"				{ TBool _ }
  "if"				        { TIf _ }
  "else"				{ TElse _ }
  "true"				{ TTrue _ }
  "false"				{ TFalse _ }
  "this"				{ TThis _ }
  "length"				{ TLength _ }
  "while"				{ TWhile _ }
  integer_literal			{ TIntLiteral _ $$ }
  ident		                        { TIdent _ $$ }
  "{"	 	 	   		{ TLeftBrace _ }
  "}"					{ TRightBrace _ }
  ","					{ TComma _ }
  "["					{ TLeftBrack _ }
  "]"					{ TRightBrack _ }
  op                                    { TOp _ $$}
  comop                                 { TComOp _ $$ }
  "("                                   { TLeftParen _ }
  ")"                                   { TRightParen _ }
  ";"                                   { TSemiColon _ }
  "."                                   { TPeriod _ }
  "!"                                   { TNot _ }
  "="                                   { TEquals _ }
  "System.out.println"                  { TPrint _ }
%%

Program : 
        MainClass ClassDeclList { Program $1 $2 }
MainClass : 
          "class" ident "{" "public" "void" "main" "(" "String" "[" "]" ident ")" "{" Statement "}" "}" { MClass $2 $11 $14 }


ClassDeclList :
          ClassDecl     { ClassDeclList $1 CEmpty }
          | ClassDecl ClassDeclList { ClassDeclList $1 $2 }
          |             { CEmpty }

ClassDecl : 
          "class" ident "{" VarDeclList MethodDeclList "}"                     { ClassDecl $2 "void" $4 $5 }
          | "class" ident "extends" ident "{" VarDeclList MethodDeclList "}"   { ClassDecl $2 $4 $6 $7 }


MethodDeclList :
     MethodDecl                   { MethodDeclList $1 MEmpty }
     | MethodDecl MethodDeclList  { MethodDeclList $1 $2 }
     |                            { MEmpty }

MethodDecl : 
     "public" Type ident "(" FormalList ")" "{" VarDeclList StatementList "return" Exp ";" "}" { MethodDecl $2 $3 $5 $8 $9 $11 }

VarDeclList :
     Type ident ";" { VarDeclList $1 $2 VEmpty }
     | Type ident ";" VarDeclList { VarDeclList $1 $2 $4 }
     |              { VEmpty }

FormalList :
     Type ident       { FormalList $1 $2 FEmpty }
     | Type ident FormalList { FormalList $1 $2 $3 }

Type :
     "int" "[" "]"    { TypeIntArray }
     | "boolean"      { TypeBoolean }
     | "int"          { TypeInt }
     | ident          { TypeIdent $1 }

Statement :
    "{" StatementList "}"                            { SList $2 }
    | "if" "(" Exp ")" Statement "else" Statement  { SIfElse $3 $5 $7 }
    | "while" "(" Exp ")" Statement                { SWhile $3 $5 }
    | "System.out.println" "(" Exp ")" ";"         { SPrint $3 }
    | ident "=" Exp ";"                              { SEqual $1 $3 }
    | ident "[" Exp "]" "=" Exp ";"                  { SArrayEqual $1 $3 $6 }

StatementList :
    Statement               { StatementList Empty $1 }
    | StatementList Statement   { StatementList $1 $2 }

Exp : 
    Exp op Exp                        { ExpOp $1 $2 $3}
    | Exp comop Exp                   { ExpComOp $1 $2 $3}
    | Exp "[" Exp "]"                 { ExpArray $1 $3}
    | Exp "." "length"                { ExpLength $1}
    | Exp "." ident "(" ExpList ")"   { ExpFCall $1 $3 $5}
    | integer_literal                 { ExpInt $1}
    | "true"                          { ExpBool True}
    | "false"                         { ExpBool False}
    | ident                           { ExpIdent $1}
    | "this"                          { ExpThis }
    | "new" "int" "[" Exp "]"         { ExpNewInt $4 }  
    | "new" ident "(" ")"             { ExpNewIdent $2}
    | "!" Exp                         { ExpNot $2}
    | "(" Exp ")"                     { ExpExp $2}

ExpList :
        Exp            { ExpListExp $1 }
        | Exp ExpRest  { ExpList $1 $2 }
        |              { ExpListEmpty }

ExpRest : 
     "," Exp      { ExpRest $2 }

{
parseError :: [Token] -> a
parseError tokenList =
  let pos = tokenPosn(head(tokenList)) 
  in error ("parse error at line " ++ show(getLineNum(pos)) ++ " and column " ++ show(getColumnNum(pos)))


data Program 
    = Program MainClass ClassDeclList
      deriving (Show, Eq)



data MainClass
    = MClass String String Statement
      deriving (Show, Eq)

data ClassDeclList
    = ClassDeclList ClassDecl ClassDeclList
    | CEmpty
  deriving (Show, Eq)

data ClassDecl = ClassDecl Ident Ident VarDeclList MethodDeclList
  deriving (Show, Eq)


data MethodDeclList
    = MethodDeclList MethodDecl MethodDeclList
    | MEmpty
    deriving (Show, Eq)
data MethodDecl
    = MethodDecl Type Ident FormalList VarDeclList StatementList Exp
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
    TypeIntArray
    | TypeBoolean
    | TypeInt
    | TypeIdent Ident
    deriving (Show, Eq)

data Statement
    = Statement String
    | SList StatementList
    | SIfElse Exp Statement Statement
    | SWhile Exp Statement
    | SPrint Exp
    | SEqual Ident Exp
    | SArrayEqual Ident Exp Exp
    | StatementError
    deriving (Show, Eq)

data StatementList
    = StatementList StatementList Statement 
    | Empty
    deriving (Show, Eq)


data Exp
    = Exp String
    | ExpOp Exp Char Exp
    | ExpComOp Exp Char Exp
    | ExpArray Exp Exp -- "Exp [ Exp ]"
    | ExpFCall Exp Ident ExpList -- Exp . Ident ( ExpList )
    | ExpInt Int
    | ExpNewInt Exp
    | ExpBool Bool -- True or False
    | ExpIdent Ident
    | ExpNewIdent Ident -- new Ident ()
    | ExpExp Exp -- Exp ( Exp )
    | ExpThis
    | ExpNot Exp
    | ExpLength Exp
    | ExpError
    deriving (Show, Eq)

data Op
     = And
     | LessThan
     | Plus
     | Minus
     | Times
     deriving (Show, Eq)

type Ident = String
type Integer_Literal = Int
data ExpList
    = ExpList Exp ExpRest
    | ExpListEmpty
    | ExpListExp Exp
    deriving (Show, Eq)
data ExpRest
    = ExpRest Exp
    deriving (Show, Eq)



main = do 
  inStr <- getContents
  let parseTree = newl (alexScanTokens2 inStr)  
  putStrLn ("parseTree: " ++ show(parseTree))
  print "done"
}