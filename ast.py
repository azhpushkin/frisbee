from dataclasses import dataclass

@dataclass
class ValidatedDataclass:
    def __validate__(self):
        for field_name, field_def in self.__dataclass_fields__.items():
            actual_type = type(getattr(self, field_name))
            if actual_type != field_def.type:
                error_msg = "Field {}.{} got {} instead of {}!"
                raise TypeError(error_msg.format(
                    self.__class__.__name__,
                    field_name,
                    actual_type,
                    field_def.type
                ))

    def __post_init__(self):
        self.__validate__()


###########################################
# data Program = Program ObjectDeclList

@dataclass
class Program(ValidatedDataclass):
    object_decl_list: BaseObjectDeclList

###########################################
# data ObjectDeclList
#     = ObjectDeclList ObjectDecl ObjectDeclList
#     | OEmpty

@dataclass
class BaseObjectDeclList(ValidatedDataclass): pass

@dataclass
class ObjectDeclList(BaseObjectDeclList):
    object_decl: BaseObjectDecl
    object_decl_list: BaseObjectDeclList

@dataclass
class OEmpty(BaseObjectDeclList): pass

###########################################
# data ObjectDecl
#     = ActiveDecl  Ident VarDeclList MethodDeclList
#     | PassiveDecl Ident VarDeclList MethodDeclList

@dataclass
class BaseObjectDecl(ValidatedDataclass): pass

@dataclass
class ActiveDecl(BaseObjectDecl):
    ident: str
    var_decl_list: BaseVarDeclList
    method_decl_list: BaseMethodDeclList

@dataclass
class PassiveDecl(BaseObjectDecl):
    ident: str
    var_decl_list: VarDeclList
    method_decl_list: BaseMethodDeclList
    

###########################################
# data MethodDeclList
#     = MethodDeclList MethodDecl MethodDeclList
#     | MEmpty

@dataclass
class BaseMethodDeclList(ValidatedDataclass): pass

@dataclass
class MethodDeclList(BaseMethodDeclList):
    pass

@dataclass
class MEmpty(BaseMethodDeclList): pass

###########################################
# data MethodDecl
#     = MethodDecl Type Ident FormalList VarDeclList StatementList

@dataclass
class MethodDecl(ValidatedDataclass):
    pass

###########################################
# data VarDeclList =
#     VarDeclList Type Ident VarDeclList
#     | VEmpty

@dataclass
class BaseVarDeclList(ValidatedDataclass): pass

@dataclass
class VarDeclList(BaseVarDeclList): pass

@dataclass
class VEmpty(BaseVarDeclList): pass
###########################################
# data FormalList = 
#     FormalList Type Ident FormalList
#     | FEmpty

@dataclass
class BaseFormalList(ValidatedDataclass):
    pass


@dataclass
class FormalList(BaseFormalList):
    pass


@dataclass
class FEmpty(BaseFormalList):
    pass

###########################################
# data Type =
#       TypeAnonymous
#     | TypeMaybe Type
#     | TypeArray Type
#     | TypeInt
#     | TypeVoid
#     | TypeBool
#     | TypeString
#     | TypeIdent Ident


@dataclass
class BaseType(ValidatedDataclass): pass

@dataclass
class TypeAnonymous(BaseType): pass

@dataclass
class TypeMaybe(BaseType): pass

@dataclass
class TypeArray(BaseType): pass

@dataclass
class TypeInt(BaseType): pass

@dataclass
class TypeVoid(BaseType): pass

@dataclass
class TypeBool(BaseType): pass

@dataclass
class TypeString(BaseType): pass

@dataclass
class TypeIdent(BaseType): pass

###########################################
# data Statement
#     = SList StatementList
#     | SIfElse Exp Statement Statement
#     | SWhile Exp Statement
#     | SReturn Exp
#     | SEqual Ident Exp
#     | SEqualField Exp Ident Exp
#     | SArrayEqual Ident Exp Exp
#     | SSendMessage Exp Ident ExpList
#     | SWaitMessage Ident Exp Ident ExpList
#     | SExp Exp

@dataclass
class BaseStatement(ValidatedDataclass): pass

@dataclass
class SList(BaseStatement): pass

@dataclass
class SIfElse(BaseStatement): pass

@dataclass
class SWhile(BaseStatement): pass

@dataclass
class SReturn(BaseStatement): pass

@dataclass
class SEqual(BaseStatement): pass

@dataclass
class SEqualField(BaseStatement): pass

@dataclass
class SArrayEqual(BaseStatement): pass

@dataclass
class SSendMessage(BaseStatement): pass

@dataclass
class SWaitMessage(BaseStatement): pass

@dataclass
class SExp(BaseStatement): pass

###########################################
# data StatementList
#     = StatementList StatementList Statement 
#     | Empty

@dataclass
class BaseStatementList(ValidatedDataclass):
    pass

@dataclass
class StatementList(BaseStatementList):
    pass

@dataclass
class Empty(BaseStatementList):
    pass

###########################################
# data Exp
#     = ExpOp Exp String Exp
#     | ExpComOp Exp String Exp
#     | ExpArrayGet Exp Exp -- "Exp [ Exp ]"
#     | ExpFCall Exp Ident ExpList -- Exp . Ident ( ExpList )
#     | ExpFieldAccess Exp Ident
#     | ExpInt Int
#     | ExpString String
#     | ExpBool Bool -- True or False
#     | ExpIdent Ident
#     | ExpNewPassive Ident ExpList -- new Ident ()
#     | ExpSpawnActive Ident ExpList -- new Ident ()
#     | ExpExp Exp -- Exp ( Exp )
#     | ExpThis
#     | ExpNot Exp

@dataclass
class BaseExp(ValidatedDataclass): pass

@dataclass
class ExpOp(BaseExp): pass

@dataclass
class ExpComOp(BaseExp): pass

@dataclass
class ExpArrayGet(BaseExp): pass

@dataclass
class ExpFCall(BaseExp): pass

@dataclass
class ExpFieldAccess(BaseExp): pass

@dataclass
class ExpInt(BaseExp): pass

@dataclass
class ExpString(BaseExp): pass

@dataclass
class ExpBool(BaseExp): pass

@dataclass
class ExpIdent(BaseExp): pass

@dataclass
class ExpNewPassive(BaseExp): pass

@dataclass
class ExpSpawnActive(BaseExp): pass

@dataclass
class ExpExp(BaseExp): pass

@dataclass
class ExpThis(BaseExp): pass

@dataclass
class ExpNot(BaseExp): pass

###########################################
# data ExpList
#     = ExpList Exp ExpList
#     | ExpListEmpty    

@dataclass
class BaseExpList(ValidatedDataclass): pass

@dataclass
class ExpList(BaseExpList): pass

@dataclass
class ExpListEmpty(BaseExpList): pass
