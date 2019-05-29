import ast_def
from environ_connect import send_initial_message

from ast_parser import load_and_parse_file


def load_file(file):
    tree = load_and_parse_file(file)

    types = get_file_types(tree)
    for object in tree.objects.get_declarations():
        object._known_types = types

    return tree


def get_file_types(tree: ast_def.Program):
    file_types = {}

    for module, types in tree.imports.get_imports().items():
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


def run_program(tree: ast_def.Program):
    types = get_file_types(tree)
    main = types['Main']

    main_proxy: ast_def.ActiveProxy = main.spawn(args=[])
    send_initial_message(main_proxy.actor_uuid, 'run', [])
