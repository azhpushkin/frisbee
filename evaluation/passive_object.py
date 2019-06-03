from __future__ import annotations

import typing
from dataclasses import dataclass, field

from . import global_conf
from .ast_def.declarations import BaseObjectDecl, BaseMethodDeclList, BaseVarDeclList, MethodDecl
from .ast_def.expressions import BaseExp, ActiveProxy


class BasePassiveObjectDeclaration(BaseObjectDecl):

    def create(self, args: typing.List[BaseExp]) -> ActiveProxy:
        return NotImplemented


@dataclass
class PassiveDecl(BasePassiveObjectDeclaration):
    name: str
    vars: BaseVarDeclList
    methods: BaseMethodDeclList

    module: str = field(default_factory=lambda: 'NOT_FOUND')

    def get_methods(self):
        methods = self.methods.get_methods()
        return {m.name: m for m in methods}

    def create(self, args: typing.List[BaseExp]) -> ExpPassiveObject:
        field_names = [name for name, type in self.vars.get_fields()]
        fields = dict(zip(field_names, args))

        new_passive = ExpPassiveObject(env=fields, module=self.module, typename=self.name)
        return new_passive


@dataclass
class ExpPassiveObject(BaseExp):
    env: typing.Dict[str, BaseExp]
    module: str
    typename: str

    @property
    def declaration(self):
        return global_conf.types_mapping[self.module][self.typename]

    def evaluate(self, ctx) -> BaseExp:
        return self

    def get_field(self, name):
        return self.env[name]

    def set_field(self, name, value):
        self.env[name] = value

    def run_method(self, name, args):
        method: MethodDecl = self.declaration.get_methods()[name]
        return method.execute(this=self, args=args)

