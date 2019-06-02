from __future__ import annotations

import typing
from dataclasses import dataclass, field

from .expressions import BaseExp, ExpVoid
from .statements import BaseStatementList
from .types import BaseType

__all__ = [
    'BaseObjectDecl',

    'BaseMethodDeclList',
    'MethodDeclList',
    'MEmpty',

    'BaseMethodDecl',
    'MethodDecl',


    'BaseVarDeclList',
    'VarDeclList',
    'VEmpty',

    'BaseFormalList',
    'FormalList',
    'FEmpty',
]


class BaseObjectDecl:
    pass


@dataclass
class BaseMethodDeclList:
    def get_methods(self):
        return NotImplemented


@dataclass
class MethodDeclList(BaseMethodDeclList):
    head: BaseMethodDecl
    tail: BaseMethodDeclList

    def get_methods(self):
        return [self.head, ] + self.tail.get_methods()


@dataclass
class MEmpty(BaseMethodDeclList):
    def get_methods(self):
        return []


@dataclass
class BaseMethodDecl:
    pass


@dataclass
class MethodDecl(BaseMethodDecl):
    type: BaseType
    name: str
    args: BaseFormalList
    statements: BaseStatementList

    def execute(self, this, args: typing.List[BaseExp]):

        field_names = [x[1] for x in self.args.get_fields()]
        initial_env = {
            name: value
            for name, value in zip(field_names, args)
        }
        import random
        x = '#' * random.randint(4, 10)
        ctx = {'this': this, 'env': initial_env}
        self.statements.run(ctx=ctx)
        return ctx.get('return', ExpVoid())


@dataclass
class BaseVarDeclList:
    def get_fields(self) -> dict:
        return NotImplemented


@dataclass
class VarDeclList(BaseVarDeclList):
    typename: BaseType
    name: str
    tail: BaseVarDeclList

    def get_fields(self):
        tail_fields = self.tail.get_fields()
        tail_fields[self.name] = self.typename
        return tail_fields


@dataclass
class VEmpty(BaseVarDeclList):
    def get_fields(self) -> dict:
        return {}




@dataclass
class BaseFormalList:
    def get_fields(self):
        return NotImplemented


@dataclass
class FormalList(BaseFormalList):
    typename: BaseType
    name: str
    tail: BaseFormalList

    def get_fields(self):
        return [(self.typename, self.name), ] + self.tail.get_fields()


@dataclass
class FEmpty(BaseFormalList):
    def get_fields(self):
        return []