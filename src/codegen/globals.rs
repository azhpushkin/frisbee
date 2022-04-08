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

impl ConstantTable {
    pub fn get_constant(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }
}

impl FunctionsTable {
    pub fn get_function_placeholder(&mut self, module: ModulePathAlias, name: String) -> usize {
        self.get_placeholder("~".join(module.alias_str(), name))
    }

    pub fn get_method_placeholder(&mut self, module: ModulePathAlias, typename: String, method: String) -> usize {
        self.get_placeholder("~".join(module.alias_str(), name))
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