import re
import subprocess
from pathlib import Path

if __name__ == '__main__':
    examples_dir = Path(__file__).parent.parent / 'examples'

    files_to_run = [
        filename 
        for filename in examples_dir.iterdir()
        if re.match(r'^.+(?<!_mod)\.frisbee$', filename.name)
    ]
    
    # Pre-compile frisbee
    subprocess.run(['cargo', 'build'])

    for filename in files_to_run:
        print(f"Running {filename}... ")
        subprocess.run(['cargo', 'run', '-q', 'cc', filename])
        subprocess.run(['cargo', 'run', '-q', 'dis', f'{filename}.bytecode'])
        

        

    
