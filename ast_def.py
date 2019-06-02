import typing
from dataclasses import dataclass, field
import multiprocessing as mp
import global_conf

from environ_connect import ActorConnector



####### Definition of Program #######




####### Definition of BaseObjectDeclList #######


####### Definition of BaseObjectDecl #######


@dataclass
class ActiveDecl(BaseObjectDecl):
    def spawn(self, args: typing.List[BaseExp]) -> ExpActiveObject:
        field_names = self.vars.get_fields().keys()
        fields = dict(zip(field_names, args))

        new_active = ExpActiveObject(env=fields, module=self.module, typename=self.name)
        return new_active.start()


@dataclass
class PassiveDecl(BaseObjectDecl):
    def create(self, args: typing.List[BaseExp]) -> ExpPassiveObject:
        field_names = self.vars.get_fields().keys()
        fields = dict(zip(field_names, args))

        new_passive = ExpPassiveObject(env=fields, module=self.module, typename=self.name)
        return new_passive


####### Definition of BaseMethodDeclList #######


####### Definition of BaseMethodDecl #######


####### Definition of BaseFormalList #######


####### Definition of BaseType #######


####### Definition of BaseStatement #######



####### Definition of BaseExp #######


# Custom values
def actor_loop(actor_obj: ExpActiveObject, event: mp.Event, assigned_id: mp.Array):
    global_conf.local_connector = ActorConnector()

    assigned_id.value = global_conf.local_connector.actor_id.encode('ascii')
    actor_obj.actor_id = global_conf.local_connector.actor_id

    event.set()

    while True:
        data = eval(global_conf.local_connector.receive_message())
        message_name, args, return_address = data['name'], data['args'], data['return']

        result = actor_obj.send_message(message_name, args)

        if return_address:
            global_conf.local_connector.return_result(return_address, result)




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

