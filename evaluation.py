import ast_def
from environ_connect import send_initial_message

from ast_parser import load_and_parse_file
from builtin_types import BuiltinTypeDeclaration, BUILTIN_TYPES


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



def load_file(module_name, types_accumulator=None):
    print('Loading', module_name)
    tree = load_and_parse_file(module_name)

    if types_accumulator is None:
        types_accumulator = {}

    typenames_to_module_map = {}

    # Updating map with import types
    for module, imported_types in tree.imports.get_imports().items():
        if module in types_accumulator:
            print('Omitting second appearance', module)
        else:
            module_types = load_file(module, types_accumulator)
            types_accumulator.update(module_types)

        for imported_type in imported_types:
            typenames_to_module_map[imported_type] = module

    types_accumulator[module_name] = {}

    # Write declarations from this module to types map
    for declaration in tree.objects.get_declarations():
        types_accumulator[module_name][declaration.name] = declaration
        typenames_to_module_map[declaration.name] = module_name

    # For each new and spawn expression update module of object according to scope
    # scope is stored in typenames_to_module_map
    for declaration in tree.objects.get_declarations():
        for method in declaration.get_methods().values():
            new_and_spawn_exprs = find_new_and_spawn(method)
            for expr in new_and_spawn_exprs:
                expr.module = typenames_to_module_map[expr.typename]

    return types_accumulator


def get_file_types(tree: ast_def.Program):
    file_types = {}

    for module, types in tree.imports.get_imports().items():
        if module in BUILTIN_TYPES:
            file_types.update({'Socket': BuiltinTypeDeclaration})
            continue

        file = f'{module}.frisbee'

        module_tree = load_file(file)
        file_types.update({
            decl.name: decl
            for decl in module_tree.objects.get_declarations()
            if decl.name in types
        })

    file_types.update({
        decl.name: decl
        for decl in tree.objects.get_declarations()
    })
    return file_types


def run_program(types: dict, main_module):
    main = types[main_module]['Main']
    ast_def.types_mapping = types

    main_proxy: ast_def.ActiveProxy = main.spawn(args=[])
    send_initial_message(main_proxy.actor_id, 'run', [])
