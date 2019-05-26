import sys
from ast_parser import load_and_parse_file
from evaluation import load_file, run_program

if len(sys.argv) < 2:
    FILE = 'example.frisbee'
else:
    FILE = sys.argv[1]

tree = load_file(FILE)

run_program(tree)
