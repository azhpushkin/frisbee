use std::collections::HashMap;

use crate::loader::generate_alias;
use crate::stdlib;
use crate::types::Type;

use super::aggregate::RawFunction;
use super::annotations::TypedFields;
use super::symbols::SymbolFunc;

pub fn is_std_function(func_name: &str) -> bool {
    stdlib::STD_FUNCTIONS.iter().any(|(k, _)| *k == func_name)
}

fn std_function_signatures() -> HashMap<&'static str, (Vec<Type>, Type)> {
    // TODO: review return types when void is done!
    HashMap::from(stdlib::STD_FUNCTIONS.map(|(k, v)| (k, v())))
}

pub fn get_std_method(t: &Type, method_name: &String) -> RawFunction {
    let mut type_methods = match t {
        Type::Bool => stdlib::STD_BOOL_METHODS.iter(),
        Type::Int => stdlib::STD_INT_METHODS.iter(),
        Type::Float => stdlib::STD_FLOAT_METHODS.iter(),
        Type::String => stdlib::STD_STRING_METHODS.iter(),
        Type::List(_) => stdlib::STD_LIST_METHODS.iter(),
        _ => panic!("Unsupported type for std method: {}", t),
    };

    let (mut args, return_type) = match type_methods.find(|(k, _)| k == method_name) {
        Some((_, v)) => v(t),
        None => panic!("Not found method {} for type {}", method_name, t),
    };
    args.insert(0, t.clone());

    RawFunction {
        name: SymbolFunc::new_std_method(t, method_name.as_str()),
        return_type: return_type.clone(),
        args: TypedFields {
            types: args.clone(),
            names: args.iter().enumerate().map(|(i, _)| (i, "".into())).collect(),
        },
        body: vec![],
        short_name: method_name.clone(),
        method_of: None,
        defined_at: generate_alias(&vec!["std".into()]),
    }
}

pub fn get_std_function_raw(name: &String) -> RawFunction {
    let (args, return_type) = &std_function_signatures()[name.as_str()];
    RawFunction {
        name: SymbolFunc::new_std_function(name.as_str()),
        return_type: return_type.clone(),
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
