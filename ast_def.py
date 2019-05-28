from __future__ import annotations

import zmq
import uuid
import typing
from dataclasses import dataclass, field
from multiprocessing  import Process, Event, Value


####### Definition of BaseProgram #######

@dataclass
class BaseProgram: pass


@dataclass
class Program(BaseProgram):
    imports: BaseImportDeclList
    objects: BaseObjectDeclList


####### Definition of BaseImportDeclList #######

@dataclass
class BaseImportDeclList:
    def get_imports(self) -> typing.Dict[str, typing.List[str]]:
        return


@dataclass
class ImportDeclList(BaseImportDeclList):
    module: str
    typenames: BaseImportIdentList
    tail: BaseImportDeclList

    def get_imports(self):
        return {self.module: self.typenames.get_names(), **self.tail.get_imports()}


@dataclass
class ImportDeclListEmpty(BaseImportDeclList):
    def get_imports(self):
        return {}


####### Definition of BaseObjectDeclList #######

@dataclass
class BaseObjectDeclList:
    def get_declarations(self) -> typing.List[BaseObjectDecl]:
        return NotImplemented


@dataclass
class ObjectDeclList(BaseObjectDeclList):
    head: BaseObjectDecl
    tail: BaseObjectDeclList

    def get_declarations(self):
        return [self.head, ] + self.tail.get_declarations()


@dataclass
class OEmpty(BaseObjectDeclList):
    def get_declarations(self):
        return []


####### Definition of BaseObjectDecl #######

@dataclass
class BaseObjectDecl:
    name: str
    vars: BaseVarDeclList
    methods: BaseMethodDeclList
    _known_types: typing.Dict[str, BaseObjectDecl] = field(default_factory=dict)

    def get_methods(self):
        methods = self.methods.get_methods()
        return {m.name: m for m in methods}


@dataclass
class ActiveDecl(BaseObjectDecl):
    def spawn(self, args: typing.List[BaseExp]) -> ExpActiveObject:
        field_names = self.vars.get_fields().keys()
        fields = dict(zip(field_names, args))

        new_active = ExpActiveObject(env=fields, declaration=self, known_types=self._known_types)
        return new_active.start()


@dataclass
class PassiveDecl(BaseObjectDecl):
    def create(self, args: typing.List[BaseExp], known_types) -> ExpPassiveObject:
        field_names = self.vars.get_fields().keys()
        fields = dict(zip(field_names, args))

        new_passive = ExpPassiveObject(env=fields, declaration=self, known_types=known_types)
        return new_passive


####### Definition of BaseMethodDeclList #######

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

    def execute(
            self,
            this: typing.Union[ExpActiveObject, ExpPassiveObject],
            args: typing.List[BaseExp],
            known_types,
    ):

        field_names = [x[1] for x in self.args.get_fields()]
        initial_env = {
            name: value
            for name, value in zip(field_names, args)
        }
        import random
        x = '#' * random.randint(4, 10)
        ctx = {'this': this, 'env': initial_env, 'types': known_types}
        self.statements.run(ctx=ctx)
        return ctx.get('return', ExpVoid())


####### Definition of BaseVarDeclList #######

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


####### Definition of BaseFormalList #######

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

####### Definition of BaseType #######

@dataclass
class BaseType:
    pass


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
class SArrayEqual(BaseStatement):
    name: str
    index: BaseExp
    expr: BaseExp


@dataclass
class SSendMessage(BaseStatement):
    object: BaseExp
    method: str
    args: BaseExpList

    def run(self, ctx):
        object = self.object.evaluate(ctx)
        object.send_message(self.method, self.args.get_exprs(ctx), return_to=None)


@dataclass
class SWaitMessage(BaseStatement):
    result_name: str
    object: BaseExp
    method: str
    args: BaseExpList

    def run(self, ctx):
        object = self.object.evaluate(ctx)
        object.send_message(self.method, self.args.get_exprs(ctx), return_to=ctx['this'].queue)
        res = object.return_queue.get()
        ctx['env'][self.result_name] = res


@dataclass
class SExp(BaseStatement):
    expr: BaseExp

    def run(self, ctx):
        self.expr.evaluate(ctx)


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
        self.head.run(ctx)
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
        index = self.index.evaluate(ctx)
        return array_exp.array[index]


@dataclass
class ExpFCall(BaseExp):
    object: BaseExp
    method: str
    args: BaseExpList

    def evaluate(self, ctx):

        object_exp = self.object.evaluate(ctx)
        args: typing.List[BaseExp] = self.args.get_exprs(ctx)
        return object_exp.run_method(self.method, args)



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
        assert isinstance(other, ExpInt), 'Not int equal!'
        return ExpBool(value=self.value == other.value)

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
        assert isinstance(other, ExpString), 'Not str equal!'
        return ExpBool(value=self.value == other.value)

    def not_equal(self, other: ExpString):
        assert isinstance(other, ExpString), 'Not str not_equal!'
        return ExpBool(value=self.value != other.value)


@dataclass
class ExpVoid(BaseExp):
    def evaluate(self, ctx):
        return self

@dataclass
class ExpBool(BaseExp):
    value: bool

    def evaluate(self, ctx) -> BaseExp:
        return self

    def equal(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int equal!'
        return ExpBool(value=self.value == other.value)

    def not_equal(self, other: ExpInt):
        assert isinstance(other, ExpInt), 'Not int not_equal!'
        return ExpBool(value=self.value != other.value)


@dataclass
class ExpIdent(BaseExp):
    name: str

    def evaluate(self, ctx) -> BaseExp:
        return ctx['env'][self.name]


@dataclass
class ExpNewPassive(BaseExp):
    typename: str
    args: BaseExpList

    def evaluate(self, ctx):
        args_expr = self.args.get_exprs(ctx)
        declaration = ctx['types'][self.typename]

        return declaration.create(args_expr, ctx['types'])


@dataclass
class ExpSpawnActive(BaseExp):
    typename: str
    args: BaseExpList

    def evaluate(self, ctx):
        args_expr = self.args.get_exprs(ctx)
        declaration = ctx['types'][self.typename]

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
        else:
            raise ValueError("No method {} of actor io".format(name))

        if return_to:
            return_to.put(ExpVoid())


# Custom values
def actor_loop(actor_obj: str, event: Event, port_value: Value):
    context = zmq.Context()
    socket = context.socket(zmq.SUB)
    selected_port = socket.bind_to_random_port('tcp://127.0.0.1')
    import time; time.sleep(0.5)

    socket.setsockopt_unicode(zmq.SUBSCRIBE, '')

    port_value.value = selected_port
    event.set()
    print(222)

    while True:
        print(1)
        topic, data = socket.recv_multipart()
        data = eval(data)

        message_name, args = data
        result = actor_obj.send_message(data['name'], data['args'])
        # if return_flag == 'true':
        #     return_queue.put(result)



@dataclass
class ExpActiveObject(BaseExp):
    env: typing.Dict[str, BaseExp]
    declaration: ActiveDecl
    known_types: typing.Dict[str, BaseObjectDecl]

    actor_uuid: str = field(default_factory=lambda: str(uuid.uuid4()))

    def start(self):
        spawned_event = Event()
        port_value = Value('d')

        proc = Process(target=actor_loop, args=(self, spawned_event, port_value))
        proc.start()
        spawned_event.wait()

        return ActiveProxy(actor_uuid=self.actor_uuid, port_value=port_value.value)

    def evaluate(self, ctx) -> BaseExp:
        return self

    def get_field(self, name):
        return self.env['name']

    def set_field(self, name, value):
        self.env['name'] = value

    def send_message(self, name, args, return_to=None):
        method: MethodDecl = self.declaration.get_methods()[name]
        return method.execute(this=self, args=args, known_types=self.known_types)


@dataclass
class ActiveProxy:
    actor_uuid: int
    port_value: int

    def __post_init__(self):
        context = zmq.Context()
        socket = context.socket(zmq.PUB)
        socket.connect('tcp://127.0.0.1:{}'.format(self.port_value))
        import time; time.sleep(0.1)
        self._socket = socket

    def evaluate(self, ctx):
        return self

    def get_field(self, name):
        raise ValueError('Cannot get field of actor!')

    def set_field(self, name, value):
        raise ValueError('Cannot set field of actor!')

    def send_message(self, name, args, return_to=None):
        flag = 'true' if return_to else 'false'
        print(123)
        self._socket.send_multipart([
            self.actor_uuid.encode('ascii'),
            str({'name': name, 'args': args}).encode('ascii')
        ])




@dataclass
class ExpPassiveObject(BaseExp):
    env: typing.Dict[str, BaseExp]
    declaration: PassiveDecl
    known_types: typing.Dict[str, BaseObjectDecl]

    def evaluate(self, ctx) -> BaseExp:
        return self

    def get_field(self, name):
        return self.env[name]

    def set_field(self, name, value):
        self.env[name] = value

    def run_method(self, name, args):
        method: MethodDecl = self.declaration.get_methods()[name]
        return method.execute(this=self, args=args, known_types=self.known_types)


@dataclass
class ExpArray(BaseExp):
    array: typing.List[typing.Any]

    def evaluate(self, cxt) -> BaseExp:
        return self


####### Definition of BaseExpList #######

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


####### Definition of BaseImportIdentList #######

@dataclass
class BaseImportIdentList:
    def get_names(self):
        return NotImplemented


@dataclass
class ImportIdentList(BaseImportIdentList):
    typename: str
    tail: BaseImportIdentList

    def get_names(self):
        return [self.typename, ] + self.tail.get_names()


@dataclass
class ImportIdentListEmpty(BaseImportIdentList):
    def get_names(self):
        return []
