from __future__ import annotations

from dataclasses import dataclass

from .. import global_conf
from .expressions import BaseExp, ExpBool, ExpArray, BaseExpList, ExpInt, ActiveProxy
from .types import BaseType

__all__ = [
    'BaseStatement',
    'SList',
    'SIfElse',
    'SWhile',
    'SReturn',
    'SEqual',
    'SEqualField',
    'SVarDeclEqual',
    'SVarDecl',
    'SArrayEqual',
    'SSendMessage',
    'SWaitMessage',
    'SExp',

    'BaseStatementList',
    'StatementList',
    'Empty',
]


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
        ctx['return'] = self.expr.evaluate(ctx)


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
        object = self.object.evaluate(ctx)
        object.set_field(self.field, self.expr.evaluate(ctx))


@dataclass
class SVarDeclEqual(BaseStatement):
    type: BaseType
    name: str
    expr: BaseExp

    def run(self, ctx):
        ctx['env'][self.name] = self.expr.evaluate(ctx)


@dataclass
class SVarDecl(BaseStatement):
    type: BaseType
    name: str

    def run(self, ctx):
        pass


@dataclass
class SArrayEqual(BaseStatement):
    name: str
    index: BaseExp
    expr: BaseExp

    def run(self, ctx):
        value = self.expr.evaluate(ctx)
        index = self.index.evaluate(ctx)

        assert isinstance(ctx['env'][self.name], ExpArray), "Not array!"
        assert isinstance(index, ExpInt), "Not int!"

        ctx['env'][self.name].array[index.value] = value


@dataclass
class SSendMessage(BaseStatement):
    object: BaseExp
    method: str
    args: BaseExpList

    def run(self, ctx):
        from ..active_object import ExpActiveObject
        object = self.object.evaluate(ctx)
        args = self.args.get_exprs(ctx)

        for i, o in enumerate(args):
            if isinstance(o, ExpActiveObject):
                args[i] = ActiveProxy(actor_id=o.actor_id, env_name=global_conf.env_name)

        if isinstance(object, ExpActiveObject):
            object = ActiveProxy(actor_id=object.actor_id, env_name=global_conf.env_name)

        object.send_message(self.method, args, return_to=None)


@dataclass
class SWaitMessage(BaseStatement):
    result_name: str
    object: BaseExp
    method: str
    args: BaseExpList

    def run(self, ctx):
        object: ActiveProxy = self.object.evaluate(ctx)
        object.send_message(
            self.method,
            self.args.get_exprs(ctx),
            return_to=global_conf.local_connector.actor_id
        )

        ctx['env'][self.result_name] = global_conf.local_connector.receive_return_value()


@dataclass
class SExp(BaseStatement):
    expr: BaseExp

    def run(self, ctx):
        self.expr.evaluate(ctx)


@dataclass
class BaseStatementList:
    def run(self, ctx):
        return NotImplemented


@dataclass
class StatementList(BaseStatementList):
    head: BaseStatement
    tail: BaseStatementList

    def run(self, ctx):
        self.head.run(ctx)
        if ctx.get('return', False):
            return
        return self.tail.run(ctx)


@dataclass
class Empty(BaseStatementList):
    def run(self, ctx):
        return ctx
