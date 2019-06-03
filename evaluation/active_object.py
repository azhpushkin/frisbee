from __future__ import annotations

import multiprocessing as mp
import typing
from dataclasses import dataclass, field

from . import global_conf
from .connector import ActorConnector
from .ast_def.declarations import BaseObjectDecl, BaseMethodDeclList, BaseVarDeclList, MethodDecl
from .ast_def.expressions import BaseExp, ActiveProxy


class BaseActiveObjectDeclaration(BaseObjectDecl):

    def spawn(self, args: typing.List[BaseExp]) -> ActiveProxy:
        return NotImplemented


@dataclass
class ActiveDecl(BaseActiveObjectDeclaration):
    name: str
    vars: BaseVarDeclList
    methods: BaseMethodDeclList

    module: str = field(default_factory=lambda: 'NOT_FOUND')

    def get_methods(self):
        methods = self.methods.get_methods()
        return {m.name: m for m in methods}

    def spawn(self, args):
        field_names = [name for name, type in self.vars.get_fields()]
        fields = dict(zip(field_names, args))

        new_active = ExpActiveObject(env=fields, module=self.module, typename=self.name)
        return new_active.start_and_return_proxy()


class BaseActiveObject:
    actor_id: str

    @staticmethod
    def _actor_loop(actor_obj: BaseActiveObject, event: mp.Event, assigned_id: mp.Array):
        global_conf.local_connector = ActorConnector()

        assigned_id.value = global_conf.local_connector.actor_id.encode('ascii')
        actor_obj.actor_id = global_conf.local_connector.actor_id

        event.set()

        actor_obj.on_start()
        while True:
            message_name, args, return_address = global_conf.local_connector.receive_message()

            result = actor_obj.proceed_message(message_name, args)
            if return_address:
                global_conf.local_connector.return_result(return_address, result)

    def start_and_return_proxy(self) -> ActiveProxy:
        spawned_event = mp.Event()
        assigned_id = mp.Array('c', 64)

        proc = mp.Process(target=self._actor_loop, args=(self, spawned_event, assigned_id))
        proc.start()
        spawned_event.wait()

        return ActiveProxy(actor_id=assigned_id.value.decode('ascii'))

    def on_start(self):
        pass

    def proceed_message(self, message_name: str, args: typing.List[BaseExp]) -> BaseExp:
        return NotImplemented


@dataclass
class ExpActiveObject(BaseActiveObject):
    env: typing.Dict[str, BaseExp]
    module: str
    typename: str

    @property
    def declaration(self):
        return global_conf.types_mapping[self.module][self.typename]

    def get_field(self, name):
        return self.env[name]

    def set_field(self, name, value):
        self.env[name] = value

    def proceed_message(self, name, args):
        method: MethodDecl = self.declaration.get_methods()[name]
        return method.execute(this=self, args=args)

    def run_method(self, name, args):
        method: MethodDecl = self.declaration.get_methods()[name]
        return method.execute(this=self, args=args)
