class BaseActiveObject:




class ExpActiveObject(BaseExp):
    env: typing.Dict[str, BaseExp]
    module: str
    typename: str

    actor_id: str = field(default_factory=lambda: None)

    @property
    def declaration(self):
        return global_conf.types_mapping[self.module][self.typename]

    def start(self):
        spawned_event = mp.Event()
        assigned_id = mp.Array('c', 64)

        proc = mp.Process(target=actor_loop, args=(self, spawned_event, assigned_id))
        proc.start()
        spawned_event.wait()

        return ActiveProxy(actor_id=assigned_id.value.decode('ascii'))

    def evaluate(self, ctx) -> BaseExp:
        return self

    def get_field(self, name):
        return self.env['name']

    def set_field(self, name, value):
        self.env['name'] = value

    def send_message(self, name, args, return_to=None):
        method: MethodDecl = self.declaration.get_methods()[name]
        return method.execute(this=self, args=args)
