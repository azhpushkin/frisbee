use std::collections::HashMap;

use crate::ast::verified::TypedFields;
use crate::types::VerifiedType;

pub struct LocalVariables {
    current_variables: HashMap<String, String>,
    all_locals: HashMap<String, VerifiedType>,
    locals_order: Vec<(String, usize)>,
    current_level: usize,
}

impl Default for LocalVariables {
    fn default() -> Self {
        Self {
            current_variables: HashMap::new(),
            all_locals: HashMap::new(),
            locals_order: vec![],
            current_level: 0,
        }
    }
}

impl LocalVariables {
    pub fn from_function_arguments(args: &TypedFields) -> Self {
        let mut new_storage = Self::default();
        for (name, argtype) in args.iter() {
            new_storage.all_locals.insert(name.clone(), argtype.clone());
            new_storage.current_variables.insert(name.clone(), name.clone());
        }
        new_storage
    }

    pub fn start_new_scope(&mut self) {
        self.current_level += 1;
    }

    pub fn drop_current_scope(&mut self) {
        self.current_level -= 1;
        loop {
            let last = self.locals_order.last();
            let is_dropped_scope = last.map(|(_, lvl)| *lvl > self.current_level).unwrap_or(false);
            if is_dropped_scope {
                let (name, _) = self.locals_order.pop().unwrap();
                self.current_variables.remove(&name).unwrap();
            } else {
                break;
            }
        }
    }

    pub fn add_variable_exact(&mut self, name: &str, t: &VerifiedType) -> Result<(), String> {
        if self.current_variables.contains_key(name) {
            return Err(format!("Variable `{}` was already defined before", name,));
        }

        self.current_variables.insert(name.into(), name.into());
        self.locals_order.push((name.into(), self.current_level));
        self.all_locals.insert(name.into(), t.clone());
        Ok(())
    }

    pub fn add_variable(&mut self, name: &str, t: &VerifiedType) -> Result<String, String> {
        if self.current_variables.contains_key(name) {
            return Err(format!("Variable `{}` was already defined before", name,));
        }

        let real_name = format!("{}_{}", name, self.all_locals.len());

        self.current_variables.insert(name.into(), real_name.clone());
        self.locals_order.push((name.into(), self.current_level));
        self.all_locals.insert(real_name.clone(), t.clone());

        Ok(real_name)
    }

    pub fn get_variable(&self, name: &str) -> Result<(VerifiedType, String), String> {
        self.current_variables
            .get(name)
            .map(|real| (self.all_locals[real].to_owned(), real.into()))
            .ok_or_else(|| format!("Variable `{}` not defined", name))
    }

    pub fn move_all_variables(self) -> HashMap<String, VerifiedType> {
        self.all_locals
    }
}
