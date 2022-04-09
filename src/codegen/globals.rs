use std::collections::HashMap;

use crate::ast::ModulePathAlias;

pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
}

pub struct ConstantTable {
    // TODO: this might be a good place to use a hashmap
    constants: Vec<Constant>,
}

pub struct FunctionsTable {
    functions: HashMap<String, usize>,
}

pub struct Globals {
    pub constants: ConstantTable,
    pub functions: FunctionsTable,
}

impl Globals {
    pub fn new() -> Self {
        Globals {
            constants: ConstantTable { constants: vec![] },
            functions: FunctionsTable { functions: HashMap::new() },
        }
    }
}

impl ConstantTable {
    pub fn get_constant(&mut self, constant: Constant) -> u8 {
        self.constants.push(constant);
        (self.constants.len() - 1) as u8
    }
}

impl FunctionsTable {
    // TODO: this join by ~ is strange, should rework I guess

    pub fn get_function_placeholder(&mut self, module: ModulePathAlias, name: String) -> usize {
        self.get_placeholder(vec![module.0, name].join("~"))
    }

    pub fn get_method_placeholder(
        &mut self,
        module: ModulePathAlias,
        typename: String,
        method: String,
    ) -> usize {
        self.get_placeholder(vec![module.0, typename, method].join("~"))
    }

    fn get_placeholder(&mut self, alias: String) -> usize {
        if let Some(placeholder) = self.functions.get(&alias) {
            *placeholder
        } else {
            let placeholder = self.functions.len();
            self.functions.insert(alias, placeholder);
            placeholder
        }
    }
}
