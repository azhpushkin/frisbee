use std::collections::HashMap;

use crate::ast::ModulePathAlias;

use super::constants::Constant;

pub struct Globals {
    pub constants: Vec<Constant>,
    pub functions: HashMap<String, usize>,
}

impl Globals {
    pub fn new() -> Self {
        Globals { constants: vec![], functions: HashMap::new() }
    }

    pub fn get_constant(&mut self, constant: Constant) -> u8 {
        self.constants.push(constant);
        (self.constants.len() - 1) as u8
    }

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
