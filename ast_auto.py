import re

class ValidateArgs:
    def __init__(self, **kwargs):
        for field in self._fields:
            setattr(self, field, kwargs[field])


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

generated_classes = []

for definition in re.findall(dataclass_regex, haskell_dataclasses):
    base_class_name, variations = definition.split('=')
    base_class_name = base_class_name.strip()
    
    base_class = type(
        'Base' + base_class_name,
        (ValidateArgs, ),
        {}
    )
    generated_classes.append(base_class)
    
    for variation in variations.split('|'):
        variation, field_names = variation.strip().split('--')
        
        words = variation.split()
        field_names = field_names.replace(' ', '').split(',')
        if field_names == ['']:
            field_names = []
        
        variation_name, fields = words[0], words[1:]
        
        camelcase_to_snakecase
        variation_class = type(
            variation_name,
            (base_class, ),
            {'_fields': field_names}
        )

        generated_classes.append(variation_class)

for klass in generated_classes:
    globals()[klass.__name__] = klass


def parse_ast_to_classes(value):
    
    if isinstance(value, str):    
        if value.startswith('"') and value.endswith('"'):
            return value.replace('"', '')
        elif value.isdigit():
            return int(value)
        elif value == 'True':
            return True
        elif value == 'False':
            return False
        elif value in globals():
            return globals()[value]
        else:
            raise ValueError(f'Unknown value {top_value}')

    top_class = globals()[value[0]]

    parsed_args = map(parse_ast_to_classes, value[1:])
    fields_kwargs = dict(zip(top_class._fields, parsed_args))
    return top_class(**fields_kwargs)
    
    

    
    


