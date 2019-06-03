from __future__ import annotations

import typing
from dataclasses import dataclass, field
from .. import global_conf

__all__ = [
    'BaseExp',
    'ExpOp',
    'ExpComOp',
    'ExpArrayGet',
    'ExpArrayValue',
    'ExpFCall',
    'ExpFieldAccess',
    'ExpInt',
    'ExpVoid',
    'ExpString',
    'ExpBool',
    'ExpIdent',
    'ExpNewPassive',
    'ExpSpawnActive',
    'ExpThis',
    'ExpNot',
    'ExpIO',
    'ExpArray',
    'ExpExp',
    'ActiveProxy',

    'BaseExpList',
    'ExpList',
    'ExpListEmpty',
]


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
        right_expr = self.right.evaluate(ctx)

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
        right_expr = self.right.evaluate(ctx)

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

    def evaluate(self, ctx):
        array_exp: ExpArray = self.array.evaluate(ctx)
        index: ExpInt = self.index.evaluate(ctx)
        assert isinstance(index, ExpInt)
        return array_exp.array[index.value]


@dataclass
class ExpArrayValue(BaseExp):
    elements: BaseExpList

    def evaluate(self, ctx):
        return ExpArray(array=self.elements.get_exprs(ctx))


@dataclass
class ExpFCall(BaseExp):
    object: BaseExp
    method: str
    args: BaseExpList

    def evaluate(self, ctx):
        object_exp = self.object.evaluate(ctx)
        args: typing.List[BaseExp] = self.args.get_exprs(ctx)
        return object_exp.run_method(name=self.method, args=args)


@dataclass
class ExpFieldAccess(BaseExp):
    object: BaseExp
    field: str

    def evaluate(self, ctx):
        object_exp = self.object.evaluate(ctx)
        return object_exp.get_field(self.field)


@dataclass
class ExpInt(BaseExp):
    value: int

    def evaluate(self, ctx) -> BaseExp:
        return self

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
        if isinstance(other, ExpInt):
            return ExpBool(value=self.value == other.value)
        elif isinstance(other, ExpInt):
            return ExpBool(value=False)
        else:
            raise ValueError('Not int equal!')

    def not_equal(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int not_equal!'
        return ExpBool(value=self.value != other.value)


@dataclass
class ExpString(BaseExp):
    value: str

    def evaluate(self, ctx) -> BaseExp:
        return self

    def add(self, other: ExpString):
        assert isinstance(other, ExpString), 'Not str added!'
        return ExpString(value=self.value + other.value)

    def equal(self, other: ExpString):
        if isinstance(other, ExpString):
            return ExpBool(value=self.value == other.value)
        elif isinstance(other, ExpVoid):
            return ExpBool(value=False)
        else:
            raise ValueError('Not str equal!')

    def not_equal(self, other: ExpString):
        assert isinstance(other, ExpString), 'Not str not_equal!'
        return ExpBool(value=self.value != other.value)


@dataclass
class ExpVoid(BaseExp):
    def evaluate(self, ctx):
        return self

    def equal(self, other):
        return ExpBool(isinstance(other, ExpVoid))


@dataclass
class ExpBool(BaseExp):
    value: bool

    def evaluate(self, ctx) -> BaseExp:
        return self

    def equal(self, other: ExpBool):
        if isinstance(other, ExpBool):
            return ExpBool(value=self.value == other.value)
        elif isinstance(other, ExpVoid):
            return ExpBool(value=False)
        else:
            raise ValueError('Not bool equal!')

    def not_equal(self, other: ExpBool):
        assert isinstance(other, ExpBool), 'Not bool!'
        return ExpBool(value=self.value != other.value)

    def andalso(self, other):
        assert isinstance(other, ExpBool), 'Not bool!'
        return ExpBool(value=self.value and other.value)

    def orelse(self, other):
        assert isinstance(other, ExpBool), 'Not bool!'
        return ExpBool(value=self.value or other.value)


@dataclass
class ExpIdent(BaseExp):
    name: str

    def evaluate(self, ctx) -> BaseExp:
        return ctx['env'][self.name]


@dataclass
class ExpNewPassive(BaseExp):
    typename: str
    args: BaseExpList
    module: str = field(default_factory=lambda: 'NOT_FOUND')

    def evaluate(self, ctx):
        # print('New', self.module, self.typename)
        args_expr = self.args.get_exprs(ctx)
        declaration = global_conf.types_mapping[self.module][self.typename]

        return declaration.create(args_expr)


@dataclass
class ExpSpawnActive(BaseExp):
    typename: str
    args: BaseExpList
    module: str = field(default_factory=lambda: 'NOT_FOUND')

    def evaluate(self, ctx):
        # print('Spawn', self.module, self.typename)
        args_expr = self.args.get_exprs(ctx)
        declaration = global_conf.types_mapping[self.module][self.typename]

        return declaration.spawn(args_expr)


@dataclass
class ExpExp(BaseExp):
    expr: BaseExp

    def evaluate(self, ctx) -> BaseExp:
        return self.expr.evaluate(ctx)


@dataclass
class ExpThis(BaseExp):
    def evaluate(self, ctx):
        return ctx['this']


@dataclass
class ExpNot(BaseExp):
    operand: BaseExp

    def evaluate(self, ctx) -> BaseExp:
        operand: ExpBool = self.operand.evaluate(ctx)

        assert isinstance(operand, ExpBool), f'Not bool applied to null but {operand}'
        return ExpBool(value=not operand.value)


@dataclass
class ExpIO(BaseExp):
    def evaluate(self, ctx):
        return self

    def send_message(self, name, args, return_to=None):
        if name == 'print':
            print("IO ACTOR CALLED: ", args)
            res = ExpVoid()
        else:
            raise ValueError("No method {} of actor io".format(name))

        if return_to:
            global_conf.local_connector.return_result(return_to, res)


@dataclass
class ExpArray(BaseExp):
    array: typing.List[typing.Any]

    def evaluate(self, cxt) -> BaseExp:
        return self

    def equal(self, other):
        if isinstance(other, ExpArray):
            return ExpBool(value = self.array == other.array)
        else:
            return ExpBool(value=False)

    def add(self, other: ExpInt):
        assert isinstance(other, ExpArray), 'Not array added!'
        return ExpArray(array=self.array + other.array)

    def run_method(self, name, args):
        if name == 'length':
            return ExpInt(value=len(self.array))
        else:
            assert False


@dataclass
class ActiveProxy(BaseExp):
    actor_id: str

    def evaluate(self, ctx):
        return self

    def send_message(self, name, args, return_to: typing.Optional[str] = None):
        global_conf.local_connector.send_message(self.actor_id, name, args, return_to)


# Definition of BaseExpList #

@dataclass
class BaseExpList:
    def get_exprs(self, ctx) -> typing.List[BaseExp]:
        return NotImplemented


@dataclass
class ExpList(BaseExpList):
    head: BaseExp
    tail: BaseExpList

    def get_exprs(self, ctx):
        head_expr = self.head.evaluate(ctx)
        return [head_expr, ] + self.tail.get_exprs(ctx)


@dataclass
class ExpListEmpty(BaseExpList):
    def get_exprs(self, ctx):
        return []
