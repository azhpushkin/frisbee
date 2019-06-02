import re
import subprocess
from pathlib import Path

from pyparsing import OneOrMore, nestedExpr

from .. import ast_def
from ..active_object import ActiveDecl
from .. passive_object import PassiveDecl

__all__ = [
    'load_and_parse_file',
]


def load_and_parse_file(path: Path) -> ast_def.Program:
    data = open(path).read().encode('utf-8')

    with subprocess.Popen('./frisbee-exe',
                          stdin=subprocess.PIPE,
                          stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE) as proc:
        out, err = proc.communicate(data)

        if proc.returncode != 0:
            err = re.sub(r'CallStack.*', '', err.decode("utf-8").replace('\n', ' '))
            err = re.sub(r'frisbee-exe: ', '', err)
            print(f'{path.stem}: {err}')
            exit()

    parser = OneOrMore(nestedExpr()).parseString
    tree = parser("(" + out.decode('ascii') + ")").asList()[0]  # top object

    ast_tree = parse_ast_to_classes(tree)
    return ast_tree


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

    if value[0] == 'ActiveDecl':
        top_class = ActiveDecl
    elif value[0] == 'PassiveDecl':
        top_class = PassiveDecl
    else:
        top_class = getattr(ast_def, value[0])

    parsed_args = map(parse_ast_to_classes, value[1:])
    fields_kwargs = dict(zip(top_class.__dataclass_fields__.keys(), parsed_args))
    return top_class(**fields_kwargs)
    


