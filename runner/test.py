import difflib
import re
import subprocess
from pathlib import Path





def check_output(filename: Path, actual_output: str):
    '''
    Example of expected section in programs:
        /*
        EXPECTED STDOUT
        ==========


        ==========
        */
    '''
    contents = open(filename).read()
    
    # .* does not match linebreaks, so use \s as well
    expected = re.search(r'==========(?P<output>.*)==========', contents, flags=re.DOTALL)
    assert expected, f"Not found expected output in {filename}"

    expected = expected.group('output').strip()
    actual_output = actual_output.strip()

    if actual_output != expected:
        d = difflib.Differ()
        diff = d.compare(expected.splitlines(), actual_output.splitlines())
        for line in diff:
            print(line)

        print(f"\n!! Output does not match expected output in {filename}!")
        exit(-1)

    




if __name__ == '__main__':
    examples_dir = Path(__file__).parent.parent / 'examples'

    files_to_run = [
        filename 
        for filename in examples_dir.iterdir()
        if re.match(r'^(?!\!).+(?<!_mod)\.frisbee$', filename.name)
    ]
    
    # Pre-compile frisbee
    subprocess.run(['cargo', 'build'])

    for filename in files_to_run:
    # for filename in ["/home/maqquettex/projects/frisbee/examples/loop.frisbee"]:
        print(f"Running {filename}... ")
        subprocess.run(['cargo', 'run', '-q', 'cc', filename])
        
        res = subprocess.run(['cargo', 'run', 'dis', f'{filename}.bytecode'], capture_output=True, text=True)
        assert res.returncode == 0, f"Error disassembling {filename}: \n{res.stderr}"

        res = subprocess.run(
            ['cargo', 'run', '-q', 'run', f'{filename}.bytecode'],
            capture_output=True, text=True
        )
        assert res.returncode == 0, f"Error running {filename}: \n{res.stderr}"

        # check_output(filename, res.stdout)
        

        

    
