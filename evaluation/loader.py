from . import ast_def
from pathlib import Path
from .connector import send_initial_message, setup_env_connection

from .ast_def.parser import load_and_parse_file
from . import global_conf
from .builtin_types import BUILTIN_TYPES


def find_new_and_spawn(dataclass_obj):
    new_and_spawn = []

    for field in dataclass_obj.__dataclass_fields__.keys():
        obj = getattr(dataclass_obj, field)

        if isinstance(obj, ast_def.ExpNewPassive) or isinstance(obj, ast_def.ExpSpawnActive):
            new_and_spawn.append(obj)

        elif hasattr(obj, '__dataclass_fields__'):
            # dataobject too, continue
            new_and_spawn.extend(find_new_and_spawn(obj))

    return new_and_spawn


def load_file(path: Path, types_accumulator=None):
    print('Loading', path.name)
    tree = load_and_parse_file(path)

    if types_accumulator is None:
        types_accumulator = {}

    typenames_to_module_map = {}

    # Updating map with import types
    for module, imported_types in tree.imports.get_imports().items():
        if module in types_accumulator:
            print('Omitting second appearance', module)
        elif module in BUILTIN_TYPES:
            types_accumulator.update({module: BUILTIN_TYPES[module]})
        else:
            module_types = load_file(path.parent / f'{module}.frisbee', types_accumulator)
            types_accumulator.update(module_types)

        for imported_type in imported_types:
            typenames_to_module_map[imported_type] = module

    types_accumulator[path.stem] = {}

    # Write declarations from this module to types map
    for declaration in tree.objects.get_declarations():
        declaration.module = path.stem
        types_accumulator[path.stem][declaration.name] = declaration
        typenames_to_module_map[declaration.name] = path.stem

    # For each new and spawn expression update module of object according to scope
    # scope is stored in typenames_to_module_map
    for declaration in tree.objects.get_declarations():
        for method in declaration.get_methods().values():
            new_and_spawn_exprs = find_new_and_spawn(method)
            for expr in new_and_spawn_exprs:
                expr.module = typenames_to_module_map[expr.typename]

    return types_accumulator


def run_program(types: dict, main_module, port):
    setup_env_connection(port)


    main = types[main_module]['Main']

    # Configure global variables
    global_conf.types_mapping = types

    main_proxy: ast_def.ActiveProxy = main.spawn(args=[])

    send_initial_message(main_proxy.actor_id, 'run', [])
