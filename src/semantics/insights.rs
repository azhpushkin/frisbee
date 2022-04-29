use std::collections::HashMap;

use crate::ast::verified::TypedFields;
use crate::types::VerifiedType;

#[derive(Debug, Clone)]
pub struct Insights {
    variables_types: HashMap<String, VerifiedType>,
    in_loop: bool,
}

impl Insights {
    pub fn new() -> Self {
        Self { variables_types: HashMap::new(), in_loop: false }
    }

    pub fn from_function_arguments(args: &TypedFields) -> Self {
        let cloned_args = args.iter().map(|(s, t)| (s.clone(), t.clone()));
        Self { variables_types: cloned_args.collect(), in_loop: false }
    }

    pub fn add_variable(&mut self, name: &str, t: &VerifiedType) -> Result<(), String> {
        if self.variables_types.contains_key(name) {
            return Err(format!("Variable {} was already defined before", name,));
        }
        self.variables_types.insert(name.into(), t.clone());
        Ok(())
    }

    pub fn get_variable(&self, name: &str) -> Result<&VerifiedType, String> {
        self.variables_types
            .get(name)
            .ok_or_else(|| format!("Variable {} not defined", name))
    }
}
