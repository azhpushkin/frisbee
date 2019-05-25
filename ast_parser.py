import ast


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
        elif value in globals():
            return globals()[value]
        else:
            raise ValueError(f'Unknown value {top_value}')

    top_class = getattr(ast, value[0])

    parsed_args = map(parse_ast_to_classes, value[1:])
    fields_kwargs = dict(zip(top_class._fields, parsed_args))
    return top_class(**fields_kwargs)
    


