import sys
from ast_parser import load_and_parse_file
from evaluation import load_file, run_program

if len(sys.argv) < 2:
    FILE = 'example.frisbee'
else:
    FILE = sys.argv[1]

main_module, _ = FILE.split('.')

module_types = load_file(main_module)

if __name__ == '__main__':
    run_program(module_types, main_module)

