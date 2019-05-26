import ast_def







def run_program(tree: ast_def.Program):
    types = {
        decl.name: decl
        for decl in tree.objects.get_declarations()
    }

    main = types['Main']
    main_spawned = main.create(args=[])
    main_spawned.run_method('run', [])



