use std::collections::HashMap;

use crate::loader::generate_alias;
use crate::stdlib;
use crate::types::Type;

use super::aggregate::RawFunction;
use super::annotations::TypedFields;
use super::symbols::SymbolFunc;

pub fn is_std_function(func_name: &str) -> bool {
    stdlib::STD_FUNCTIONS.iter().find(|(k, _)| *k == func_name).is_some()
}

fn std_function_signatures() -> HashMap<&'static str, (Vec<Type>, Type)> {
    // TODO: review return types when void is done!
    HashMap::from(stdlib::STD_FUNCTIONS.map(|(k, v)| (k, v())))
}

pub fn get_std_function_raw(name: &String) -> RawFunction {
    let (args, return_type) = &std_function_signatures()[name.as_str()];
    RawFunction {
        name: SymbolFunc::new_std_function(name.as_str()),
        return_type: Some(return_type.clone()),
        args: TypedFields {
            types: args.clone(),
            names: args.iter().enumerate().map(|(i, _)| (i, "".into())).collect(),
        },
        body: vec![],
        short_name: name.clone(),
        method_of: None,
        defined_at: generate_alias(&vec!["std".into()]),
    }
}
