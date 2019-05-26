
class ActiveObject:
    def __init__(self, fields):
        self.fields = declaration.vars.get_fields()
        self.declaration = declaration
