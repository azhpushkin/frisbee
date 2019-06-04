import sys
from pathlib import Path

from evaluation.loader import load_file, run_program

FILE = sys.argv[1]
PORT = sys.argv[2]

path = Path(sys.argv[1]).resolve()
tree = load_file(path)

if __name__ == '__main__':
    run_program(tree, main_module=path.stem, port=PORT)
