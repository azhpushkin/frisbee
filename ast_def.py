from __future__ import annotations

from dataclasses import dataclass


####### Definition of BaseProgram #######

@dataclass
class BaseProgram: pass


@dataclass
class Program(BaseProgram):
    imports: BaseImportDeclList
    objects: BaseObjectDeclList


####### Definition of BaseImportDeclList #######

@dataclass
class BaseImportDeclList: pass


@dataclass
class ImportDeclList(BaseImportDeclList):
    module: str
    typenames: BaseImportIdentList
    tail: BaseImportDeclList


@dataclass
class ImportDeclListEmpty(BaseImportDeclList): pass


####### Definition of BaseObjectDeclList #######

@dataclass
class BaseObjectDeclList: pass


@dataclass
class ObjectDeclList(BaseObjectDeclList):
    head: BaseObjectDecl
    tail: BaseObjectDeclList


@dataclass
class OEmpty(BaseObjectDeclList): pass


####### Definition of BaseObjectDecl #######

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


####### Definition of BaseMethodDeclList #######

@dataclass
class BaseMethodDeclList: pass


@dataclass
class MethodDeclList(BaseMethodDeclList):
    head: BaseMethodDecl
    tail: BaseMethodDeclList


@dataclass
class MEmpty(BaseMethodDeclList): pass


####### Definition of BaseMethodDecl #######

@dataclass
class BaseMethodDecl: pass


@dataclass
class MethodDecl(BaseMethodDecl):
    type: BaseType
    name: str
    args: BaseFormalList
    vars: BaseVarDeclList
    statements: BaseStatementList


####### Definition of BaseVarDeclList #######

@dataclass
class BaseVarDeclList: pass


@dataclass
class VarDeclList(BaseVarDeclList):
    typename: BaseType
    name: str
    tail: BaseVarDeclList


@dataclass
class VEmpty(BaseVarDeclList): pass


####### Definition of BaseFormalList #######

@dataclass
class BaseFormalList: pass


@dataclass
class FormalList(BaseFormalList):
    typename: BaseType
    name: str
    tail: BaseFormalList


@dataclass
class FEmpty(BaseFormalList): pass


####### Definition of BaseType #######

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


####### Definition of BaseStatement #######

@dataclass
class BaseStatement:
    def run(self, ctx) -> None:
        pass


@dataclass
class SList(BaseStatement):
    statements: BaseStatementList

    def run(self, ctx):
        self.statements.run(ctx)


@dataclass
class SIfElse(BaseStatement):
    condition: BaseExp
    if_body: BaseStatement
    else_body: BaseStatement

    def run(self, ctx):
        res: ExpBool = self.condition.evaluate(ctx)
        if res.value:
            self.if_body.run(ctx)
        else:
            self.else_body.run(ctx)


@dataclass
class SWhile(BaseStatement):
    condition: BaseExp
    body: BaseStatement

    def run(self, ctx):
        while self.condition.evaluate(ctx).value:
            self.body.run(ctx)

        return ctx


@dataclass
class SReturn(BaseStatement):
    expr: BaseExp

    def run(self, ctx):
        raise NotImplementedError("Implement this!")


@dataclass
class SEqual(BaseStatement):
    name: str
    expr: BaseExp

    def run(self, ctx):
        ctx['env'][self.name] = self.expr.evaluate(ctx)


@dataclass
class SEqualField(BaseStatement):
    object: BaseExp
    field: str
    expr: BaseExp

    def run(self, ctx):
        # наверно тут стоит получить имя, а потом что-то менять в ctx
        raise NotImplementedError('DO THIS')


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


####### Definition of BaseStatementList #######

@dataclass
class BaseStatementList:
    def run(self, ctx):
        return NotImplemented


@dataclass
class StatementList(BaseStatementList):
    head: BaseStatement
    tail: BaseStatementList

    def run(self, ctx):
        ctx = self.head.run(ctx)
        return self.tail.run(ctx)


@dataclass
class Empty(BaseStatementList):
    def run(self, ctx):
        return ctx


####### Definition of BaseExp #######

@dataclass
class BaseExp:
    def evaluate(self, ctx) -> BaseExp:
        pass


@dataclass
class ExpOp(BaseExp):
    left: BaseExp
    operator: str
    right: BaseExp

    def evaluate(self, ctx):
        left_expr = self.left.evaluate(ctx)
        right_expr = self.left.evaluate(ctx)

        if self.operator == '+':
            return left_expr.add(right_expr)
        elif self.operator == '-':
            return left_expr.minus(right_expr)
        elif self.operator == '*':
            return left_expr.mul(right_expr)
        elif self.operator == '/':
            return left_expr.divide(right_expr)
        elif self.operator == 'and':
            return left_expr.andalso(right_expr)
        elif self.operator == 'or':
            return left_expr.orelse(right_expr)
        else:
            raise ValueError('Unknown operator {}'.format(self.operator))


@dataclass
class ExpComOp(BaseExp):
    left: BaseExp
    operator: str
    right: BaseExp

    def evaluate(self, ctx):
        left_expr = self.left.evaluate(ctx)
        right_expr = self.left.evaluate(ctx)

        if self.operator == '<':
            return left_expr.less(right_expr)
        elif self.operator == '>':
            return left_expr.greater(right_expr)
        elif self.operator == '==':
            return left_expr.equal(right_expr)
        elif self.operator == '!=':
            return left_expr.not_equal(right_expr)
        else:
            raise ValueError('Unknown compare operator {}'.format(self.operator))


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

    def add(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int added!'
        return ExpInt(value=self.value + other.value)

    def minus(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int minused!'
        return ExpInt(value=self.value - other.value)

    def divide(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int divided!'
        return ExpInt(value=self.value // other.value)

    def mul(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int muled!'
        return ExpInt(value=self.value * other.value)

    def less(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int less!'
        return ExpBool(value=self.value < other.value)

    def greater(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int greater!'
        return ExpBool(value=self.value > other.value)

    def equal(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int equal!'
        return ExpBool(value=self.value == other.value)

    def not_equal(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int not_equal!'
        return ExpBool(value=self.value != other.value)


@dataclass
class ExpString(BaseExp):
    value: str

    def add(self, other: ExpString):
        assert isinstance(other, ExpString), 'Not str added!'
        return ExpString(value=self.value + other.value)

    def equal(self, other: ExpString):
        assert isinstance(other, ExpString), 'Not str equal!'
        return ExpBool(value=self.value == other.value)

    def not_equal(self, other: ExpString):
        assert isinstance(other, ExpString), 'Not str not_equal!'
        return ExpBool(value=self.value != other.value)


@dataclass
class ExpBool(BaseExp):
    value: bool

    def equal(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int equal!'
        return ExpBool(value=self.value == other.value)

    def not_equal(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int not_equal!'
        return ExpBool(value=self.value != other.value)


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


####### Definition of BaseExpList #######

@dataclass
class BaseExpList: pass


@dataclass
class ExpList(BaseExpList):
    head: BaseExp
    tail: BaseExpList


@dataclass
class ExpListEmpty(BaseExpList): pass


####### Definition of BaseImportIdentList #######

@dataclass
class BaseImportIdentList: pass


@dataclass
class ImportIdentList(BaseImportIdentList):
    typename: str
    tail: BaseImportIdentList


@dataclass
class ImportIdentListEmpty(BaseImportIdentList): pass
