import subprocess
from pyparsing import OneOrMore, nestedExpr

from ast_auto import *



FILE = 'example.frisbee'

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
