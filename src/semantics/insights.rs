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

#[derive(Debug, Clone)]
pub struct Insights {
    pub is_in_loop: bool,
    pub return_found: bool,
    pub uninitialized_variables: HashSet<String>,
}

impl Insights {
    pub fn new() -> Self {
        Self {
            is_in_loop: false,
            return_found: false,
            uninitialized_variables: HashSet::new(),
        }
    }

    pub fn merge_with(&mut self, other: Insights) {
        if self.is_in_loop != other.is_in_loop {
            panic!("Different is_in_loop values should not occur!");
        }
        self.return_found &= other.return_found;

        // If variable might be unitialized due to another insights -
        // then it can be initialized in this one
        self.uninitialized_variables
            .extend(other.uninitialized_variables.into_iter());
    }

    pub fn is_uninitialized(&self, name: &str) -> bool {
        self.uninitialized_variables.contains(name)
    }

    pub fn add_uninitialized(&mut self, name: &str) {
        self.uninitialized_variables.insert(name.into());
    }

    pub fn mark_as_initialized(&mut self, name: &str) {
        self.uninitialized_variables.remove(name);
    }
}

macro_rules! with_insights_as_in_loop {
    ($insights:ident, $code:block) => {{
        let before = $insights.is_in_loop;
        $insights.is_in_loop = true;
        let res = $code;
        $insights.is_in_loop = before;
        res
    }};
}

pub(super) use with_insights_as_in_loop;
