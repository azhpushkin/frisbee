import difflib
import re
import sys
import subprocess as sp
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

    if expected.startswith('[UNORDERED]'):
        expected = set(expected.replace('[UNORDERED]', '').strip().split('\n'))
        actual_output = set(actual_output.split('\n'))
        if expected != actual_output:
            print('Extra lines: ', actual_output - expected)
            print('Missing lines: ', expected - actual_output)
            exit(-1)
    elif actual_output != expected:
        d = difflib.Differ()
        diff = d.compare(expected.splitlines(), actual_output.splitlines())
        for line in diff:
            print(line)

        print(f"\n!! Output does not match expected output in {filename}!")
        exit(-1)


def get_input(filename: Path):
    contents = open(filename).read()
    
    # .* does not match linebreaks, so use \s as well
    match = re.search(r'// INPUT: (?P<input>.*)\n', contents)
    if not match:
        return None
    match = match.group('input').strip()
    return '\n'.join(match.split('~')) + '\n'


def run_file(filename):
    print(f"Running {filename}... ")
    res = sp.run(['cargo', 'run', '-q', 'cc', filename])
    assert res.returncode == 0, f"Error compiling {filename}"
    
    res = sp.run(['cargo', 'run', 'dis', f'{filename}.bytecode'], capture_output=True, text=True)
    assert res.returncode == 0, f"Error disassembling {filename}: \n{res.stderr}"

    file_input = get_input(filename)
    
    res = sp.run(
        ['cargo', 'run', '-q', 'run', f'{filename}.bytecode'],
        capture_output=True,
        text=True,
        input=file_input,
        timeout=5  # 5 seconds is enough to determine infinite loop 
    )
    assert res.returncode == 0, f"Error running {filename}: \n{res.stderr}"

    check_output(filename, res.stdout)


if __name__ == '__main__':
    if len(sys.argv) == 1:
        examples_dir = Path(__file__).parent.parent / 'examples'

        files_to_run = [
            filename 
            for filename in examples_dir.iterdir()
            if re.match(r'^(?!\!).+(?<!_mod)\.frisbee$', filename.name)
        ]
    else:
        files_to_run = sys.argv[1:]
    
    # Pre-compile frisbee
    sp.run(['cargo', 'build'])

    for filename in files_to_run:
        run_file(filename)
