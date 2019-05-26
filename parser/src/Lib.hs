module Lib
    ( printTree
    ) where

import Frisbee
import Tokens

import Text.Pretty.Simple (pPrint, pPrintNoColor)

parseText = frisbee . alexScanTokens2

printTree :: IO ()
printTree = do
    inStr <- getContents
    -- pPrintNoColor $ alexScanTokens2 inStr
    pPrintNoColor $ parseText inStr


