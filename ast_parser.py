import ast_def


def parse_ast_to_classes(value):
    
    if isinstance(value, str):    
        if value.startswith('"') and value.endswith('"'):
            return value.replace('"', '')
        elif value.isdigit():
            return int(value)
        elif value == 'True':
            return True
        elif value == 'False':
            return False
        elif hasattr(ast_def, value):
            return getattr(ast_def, value)()
        else:
            raise ValueError(f'Unknown value {value}')

    top_class = getattr(ast_def, value[0])

    parsed_args = map(parse_ast_to_classes, value[1:])
    fields_kwargs = dict(zip(top_class.__dataclass_fields__.keys(), parsed_args))
    return top_class(**fields_kwargs)
    


