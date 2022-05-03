use std::collections::{HashMap, HashSet};

use crate::ast::verified::TypedFields;
use crate::types::VerifiedType;

#[derive(Debug, Clone)]
pub struct Insights {
    pub is_in_loop: bool,
    pub return_found: bool,
    pub break_or_continue_found: bool,
    pub uninitialized_variables: HashSet<String>,
    pub initialized_own_fields: HashSet<String>,
}

impl Insights {
    pub fn new() -> Self {
        Self {
            is_in_loop: false,
            return_found: false,
            break_or_continue_found: false,
            uninitialized_variables: HashSet::new(),
            initialized_own_fields: HashSet::new(),
        }
    }

    pub fn merge_with(&mut self, other: Insights) {
        let Insights { uninitialized_variables, initialized_own_fields, .. } = other;

        if self.is_in_loop != other.is_in_loop {
            panic!("Different is_in_loop values should not occur!");
        }
        // For break and continue it is important that they MIGHT occur, so we check
        // if either one of insights have encountered it
        self.break_or_continue_found |= other.break_or_continue_found;

        // For return it is important that it MUST occur, so we check that
        // both insights have found return statement
        self.return_found &= other.return_found;

        // If variable я might be unitialized due to another insights -
        // then it can be initialized in this one
        self.uninitialized_variables
            .extend(uninitialized_variables.into_iter());

        self.initialized_own_fields
            .retain(|e| initialized_own_fields.contains(e));
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

    pub fn mark_own_field_as_initialized(&mut self, field: &str) {
        self.initialized_own_fields.insert(field.into());
    }
}
