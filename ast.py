
from dataclasses import dataclass


@dataclass
class BaseProgram: pass

@dataclass
class Program(BaseProgram):
    imports: BaseImportDeclList
    objects: BaseObjectDeclList

@dataclass
class BaseImportDeclList: pass

@dataclass
class ImportDeclList(BaseImportDeclList):
    module: str
    typenames: BaseImportIdentList
    tail: BaseImportDeclList

@dataclass
class ImportDeclListEmpty(BaseImportDeclList): pass

@dataclass
class BaseObjectDeclList: pass

@dataclass
class ObjectDeclList(BaseObjectDeclList):
    head: BaseObjectDecl
    tail: BaseObjectDeclList

@dataclass
class OEmpty(BaseObjectDeclList): pass

@dataclass
class BaseObjectDecl: pass

@dataclass
class ActiveDecl(BaseObjectDecl):
    name: str
    vars: BaseVarDeclList
    methods: BaseMethodDeclList

@dataclass
class PassiveDecl(BaseObjectDecl):
    name: str
    vars: BaseVarDeclList
    methods: BaseMethodDeclList

@dataclass
class BaseMethodDeclList: pass

@dataclass
class MethodDeclList(BaseMethodDeclList):
    head: BaseMethodDecl
    tail: BaseMethodDeclList

@dataclass
class MEmpty(BaseMethodDeclList): pass

@dataclass
class BaseMethodDecl: pass

@dataclass
class MethodDecl(BaseMethodDecl):
    type: BaseType
    name: str
    args: BaseFormalList
    vars: BaseVarDeclList
    statements: BaseStatementList

@dataclass
class BaseVarDeclList: pass

@dataclass
class VarDeclList(BaseVarDeclList):
    typename: BaseType
    name: str
    tail: BaseVarDeclList

@dataclass
class VEmpty(BaseVarDeclList): pass

@dataclass
class BaseFormalList: pass

@dataclass
class FormalList(BaseFormalList):
    typename: BaseType
    name: str
    tail: BaseFormalList

@dataclass
class FEmpty(BaseFormalList): pass

@dataclass
class BaseType: pass

@dataclass
class TypeAnonymous(BaseType): pass

@dataclass
class TypeMaybe(BaseType):
    type: BaseType

@dataclass
class TypeArray(BaseType):
    type: BaseType

@dataclass
class TypeInt(BaseType): pass

@dataclass
class TypeVoid(BaseType): pass

@dataclass
class TypeBool(BaseType): pass

@dataclass
class TypeString(BaseType): pass

@dataclass
class TypeIdent(BaseType):
    name: str

@dataclass
class BaseStatement: pass

@dataclass
class SList(BaseStatement):
    statements: BaseStatementList

@dataclass
class SIfElse(BaseStatement):
    condition: BaseExp
    if_body: BaseStatement
    else_body: BaseStatement

@dataclass
class SWhile(BaseStatement):
    condition: BaseExp
    body: BaseStatement

@dataclass
class SReturn(BaseStatement):
    expr: BaseExp

@dataclass
class SEqual(BaseStatement):
    name: str
    expr: BaseExp

@dataclass
class SEqualField(BaseStatement):
    object: BaseExp
    field: str
    expr: BaseExp

@dataclass
class SArrayEqual(BaseStatement):
    name: str
    index: BaseExp
    expr: BaseExp

@dataclass
class SSendMessage(BaseStatement):
    object: BaseExp
    method: str
    args: BaseExpList

@dataclass
class SWaitMessage(BaseStatement):
    result_name: str
    object: BaseExp
    method: str
    args: BaseExpList

@dataclass
class SExp(BaseStatement):
    expr: BaseExp

@dataclass
class BaseStatementList: pass

@dataclass
class StatementList(BaseStatementList):
    head: BaseStatementList
    tail: BaseStatement

@dataclass
class Empty(BaseStatementList): pass

@dataclass
class BaseExp: pass

@dataclass
class ExpOp(BaseExp):
    left: BaseExp
    operator: str
    right: BaseExp

@dataclass
class ExpComOp(BaseExp):
    left: BaseExp
    operator: str
    right: BaseExp

@dataclass
class ExpArrayGet(BaseExp):
    array: BaseExp
    index: BaseExp

@dataclass
class ExpFCall(BaseExp):
    object: BaseExp
    method: str
    args: BaseExpList

@dataclass
class ExpFieldAccess(BaseExp):
    object: BaseExp
    field: str

@dataclass
class ExpInt(BaseExp):
    value: int

@dataclass
class ExpString(BaseExp):
    value: str

@dataclass
class ExpBool(BaseExp):
    value: bool

@dataclass
class ExpIdent(BaseExp):
    name: str

@dataclass
class ExpNewPassive(BaseExp):
    typename: str
    args: BaseExpList

@dataclass
class ExpSpawnActive(BaseExp):
    typename: str
    args: BaseExpList

@dataclass
class ExpExp(BaseExp):
    expr: BaseExp

@dataclass
class ExpThis(BaseExp): pass

@dataclass
class ExpIO(BaseExp): pass

@dataclass
class ExpNot(BaseExp):
    operand: BaseExp

@dataclass
class BaseExpList: pass

@dataclass
class ExpList(BaseExpList):
    head: BaseExp
    tail: BaseExpList

@dataclass
class ExpListEmpty(BaseExpList): pass

@dataclass
class BaseImportIdentList: pass

@dataclass
class ImportIdentList(BaseImportIdentList):
    typename: str
    tail: BaseImportIdentList

@dataclass
class ImportIdentListEmpty(BaseImportIdentList): pass
