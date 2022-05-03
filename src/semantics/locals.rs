use std::collections::{HashMap, HashSet};

use crate::ast::verified::TypedFields;
use crate::types::VerifiedType;

pub struct LocalVariables {
    pub variables_types: HashMap<String, VerifiedType>,
    pub locals_order: Vec<String>, // TODO: reference here?
}

impl LocalVariables {
    pub fn from_function_arguments(args: &TypedFields) -> Self {
        let cloned_args = args.iter().map(|(s, t)| (s.clone(), t.clone()));
        Self { variables_types: cloned_args.collect(), locals_order: vec![] }
    }

    pub fn add_variable(&mut self, name: &str, t: &VerifiedType) -> Result<(), String> {
        if self.variables_types.contains_key(name) {
            return Err(format!("Variable `{}` was already defined before", name,));
        }

        self.variables_types.insert(name.into(), t.clone());
        self.locals_order.push(name.into());

        Ok(())
    }

    pub fn get_variable(&self, name: &str) -> Result<&VerifiedType, String> {
        self.variables_types
            .get(name)
            .ok_or_else(|| format!("Variable `{}` not defined", name))
    }

    pub fn drop_last_local(&mut self) -> String {
        let last_local = self.locals_order.pop().expect("Called while no locals");
        self.variables_types.remove(&last_local);
        last_local
    }

    pub fn peek_last_local(&self) -> Option<&String> {
        return self.locals_order.last();
    }

    
}