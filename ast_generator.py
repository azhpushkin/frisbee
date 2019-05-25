import re


starting_template = """
from dataclasses import dataclass

"""


class_template = """
@dataclass
class {classname}{inherit}:"""

def create_class_template(classname, inherit=None, fields=None):
    inherit = f'({inherit})' if inherit else ''
    res = class_template.format(classname=classname, inherit=inherit)
    if fields:
        for name, type_name in fields:
            types_mapping = {'String': 'str', 'Int': 'int', 'Bool': 'bool'}
            python_type = types_mapping.get(type_name, 'Base' + type_name)
            res += f'\n    {name}: {python_type}'
    else:
        res += ' pass'
    
    return res + '\n'


def generate_ast_classes():
    file = open('./parser/src/Frisbee.y').read()
    dataclass_section_regex = re.compile(
        r".*-- PYTHON START HERE"
        "(.*)"
        "-- PYTHON END HERE.*",
        re.IGNORECASE | re.DOTALL | re.MULTILINE
    )
    haskell_dataclasses = re.findall(dataclass_section_regex, file)[0]


    dataclass_regex = re.compile(
        r"data (.*?) deriving",
        re.IGNORECASE | re.DOTALL | re.MULTILINE
    )


    file = open('ast.py', 'w')
    file.write(starting_template)

    for definition in re.findall(dataclass_regex, haskell_dataclasses):
        base_class_name, variations = definition.split('=')
        base_class_name = 'Base' + base_class_name.strip()

        file.write(create_class_template(base_class_name))
        
        for variation in variations.split('|'):
            variation, field_names = variation.strip().split('--')
            
            words = variation.split()
            field_names = field_names.replace(' ', '').split(',')
            if field_names == ['']:
                field_names = []
            
            variation_class_name, fields = words[0], words[1:]
            file.write(create_class_template(
                variation_class_name,
                base_class_name,
                fields=[
                    (field_name, annotation)
                    for field_name, annotation
                    in zip(field_names, fields)
                ]
            ))
            
            
if __name__ == '__main__':
    generate_ast_classes()
