import subprocess
import sys
from pyparsing import OneOrMore, nestedExpr

from ast_parser import parse_ast_to_classes
from evaluation import run_program

if len(sys.argv) < 2:
    FILE = 'example.frisbee'
else:
    FILE = sys.argv[1]


data = open(FILE).read().encode('utf-8')

with subprocess.Popen('./frisbee-exe',
                      stdin=subprocess.PIPE,
                      stdout=subprocess.PIPE,
                      stderr=subprocess.PIPE) as proc:
    out, _ = proc.communicate(data)

    if proc.returncode != 0:
        print('ERROR: ', out)
        exit()

parser = OneOrMore(nestedExpr()).parseString
tree = parser("(" + out.decode('ascii') + ")").asList()[0]  # top object

ast_tree = parse_ast_to_classes(tree)

run_program(ast_tree)
