import sys
from pathlib import Path

from evaluation.loader import load_file, run_program

if len(sys.argv) < 2:
    FILE = 'example.frisbee'
else:
    FILE = sys.argv[1]

path = Path(sys.argv[1]).resolve()
tree = load_file(path)

if __name__ == '__main__':
    run_program(tree, main_module=path.stem)
