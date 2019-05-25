
{
module Tokens where
}

%wrapper "posn"

$digit = 0-9			-- digits
$alpha = [a-zA-Z]		-- alphabetic characters
$loweralpha = [a-z]
$upperalpha = [A-Z]
$graphic    = [$printable $white]

@string     = \" ($graphic # \")* \"



tokens :-

  $white+				;
  "active"				{ \p s -> TActive p }
  "passive"				{ \p s -> TPassive p }
  "new"					{ \p s -> TNew p }
  "spawn"					{ \p s -> TSpawn p }
  "import"					{ \p s -> TImport p }
  "from"					{ \p s -> TFrom p }
  
  "return"                              { \p s -> TReturn p }
  "def"				{ \p s -> TDef p }
  
  "Void"				{ \p s -> TVoid p }
  "val"                                 { \p s -> TVal p }
  "String"				{ \p s -> TString p }
  "Int"					{ \p s -> TInt p }
  "Bool"				{ \p s -> TBool p }
  $upperalpha[$alpha $digit]*		{ \p s -> TTypeIdent p s }
  "?"					{ \p s -> TMaybe p }
  
  "io"                                   { \p s -> TIo p }
  "if"					{ \p s -> TIf p }
  "else"				{ \p s -> TElse p }
  "true"				{ \p s -> TTrue p }
  "false"				{ \p s -> TFalse p }
  "this"				{ \p s -> TThis p }
  "while"				{ \p s -> TWhile p }
  $digit+				{ \p s -> TIntLiteral p (read s) }
  "."                                   { \p s -> TPeriod p }

  "and"					{ \p s -> TOp p "and" }
  "not"					{ \p s -> TNot p }
  "or"					{ \p s -> TOp p "or" }
  
  [\+\-\*\/]                            { \p s -> TOp p [head s] }

  "<"                                   { \p s -> TComOp p "<" }
  ">"                                   { \p s -> TComOp p ">" }
  "=="                                   { \p s -> TComOp p "==" }
  "!="                                   { \p s -> TComOp p "!=" }

  "="					{ \p s -> TEquals p }
  "<="					{ \p s -> TWaitMessage p }
  "!"					{ \p s -> TSendMessage p }

  ";" 					{ \p s -> TSemiColon p }
  "("					{ \p s -> TLeftParen p }
  ")"					{ \p s -> TRightParen p }
  $loweralpha[$alpha $digit \_ \']*		{ \p s -> TIdent p s }
  @string 	       	  		{ \p s -> TStringLiteral p (init (tail s)) -- remove the leading and trailing double quotes }
  "{"	 	 	   		{ \p s -> TLeftBrace p }
  "}"					{ \p s -> TRightBrace p }
  ","					{ \p s -> TComma p }
  "["					{ \p s -> TLeftBrack p }
  "]"					{ \p s -> TRightBrack p }
{
-- Each action has type ::AlexPosn -> String -> Token

-- The token type:
data Token =
     	TLeftBrace AlexPosn	       |
	TRightBrace AlexPosn	       |
	TComma AlexPosn		       |
	TLeftBrack AlexPosn	       |
	TRightBrack AlexPosn	       |
	TActive AlexPosn 	       |
        TPassive AlexPosn 	       |
	TDef AlexPosn	       |
	TString AlexPosn	       |
        TVal AlexPosn	       |
	TVoid AlexPosn		       |
	TInt AlexPosn		       |
	TBool AlexPosn		       |
        TIo AlexPosn		       |
	TIf AlexPosn		       |
	TElse AlexPosn		       |
	TTrue AlexPosn		       |
	TFalse AlexPosn		       |
	TThis AlexPosn		       |
	TWhile AlexPosn		       |
	TNew AlexPosn		       |
        TSpawn AlexPosn		       |
        TImport AlexPosn		       |
        TFrom AlexPosn		       |
	TOp AlexPosn String              |
	TComOp AlexPosn String           |
        TMaybe AlexPosn                  |
        TNot AlexPosn                  |
	TEquals AlexPosn               |
        TWaitMessage AlexPosn               |
        TSendMessage AlexPosn               |
	TPeriod AlexPosn               |
	TSemiColon AlexPosn            |
	TLeftParen AlexPosn 	       |
	TRightParen AlexPosn 	       |
	TIdent AlexPosn String	       |
        TTypeIdent AlexPosn String	       |
	TIntLiteral AlexPosn Int       |
	TStringLiteral AlexPosn String |
        TReturn AlexPosn                    
	deriving (Eq,Show)




tokenPosn (TLeftBrace p) = p	       
tokenPosn (TRightBrace p) = p	       
tokenPosn (TComma p) = p	       
tokenPosn (TLeftBrack p) = p	       
tokenPosn (TRightBrack p) = p	       
tokenPosn (TActive p) = p 	       
tokenPosn (TPassive p) = p 	       
tokenPosn (TDef p) = p	       
tokenPosn (TString p) = p	       
tokenPosn (TVal p) = p	       
tokenPosn (TVoid p) = p	       
tokenPosn (TInt p) = p		       
tokenPosn (TBool p) = p	       
tokenPosn (TIo p) = p		       
tokenPosn (TIf p) = p		       
tokenPosn (TElse p) = p	       
tokenPosn (TTrue p) = p	       
tokenPosn (TFalse p) = p	       
tokenPosn (TThis p) = p	       
tokenPosn (TWhile p) = p	       
tokenPosn (TNew p) = p		       
tokenPosn (TSpawn p) = p		
tokenPosn (TImport p) = p
tokenPosn (TFrom p) = p		       
tokenPosn (TOp p c) = p            
tokenPosn (TComOp p c) = p         
tokenPosn (TMaybe p) = p                
tokenPosn (TNot p) = p                
tokenPosn (TEquals p) = p             
tokenPosn (TWaitMessage p) = p             
tokenPosn (TSendMessage p) = p             
tokenPosn (TPeriod p) = p             
tokenPosn (TSemiColon p) = p          
tokenPosn (TLeftParen p) = p 	       
tokenPosn (TRightParen p) = p 	       
tokenPosn (TIdent p id) = p      
tokenPosn (TTypeIdent p id) = p      
tokenPosn (TIntLiteral p i) = p
tokenPosn (TStringLiteral p str) = p
tokenPosn (TReturn p) = p                    


getLineNum :: AlexPosn -> Int
getLineNum (AlexPn offset lineNum colNum) = lineNum 

getColumnNum :: AlexPosn -> Int
getColumnNum (AlexPn offset lineNum colNum) = colNum

alexScanTokens :: String -> [Token]
alexScanTokens2 str = go (alexStartPos,'\n',[], str)
  where go (pos,x, [], str) =
          case alexScan (pos, x, [], str) 0 of
                AlexEOF -> []
                AlexError _ -> error ("lexical error @ line " ++ show (getLineNum(pos)) ++ " and column " ++ show (getColumnNum(pos)))
                AlexSkip  inp' len     -> go inp'
                AlexToken inp' len act -> act pos (take len str) : go inp'


}