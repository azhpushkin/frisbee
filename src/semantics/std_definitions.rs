use std::collections::HashMap;

use crate::alias::ModuleAlias;
use crate::ast::verified::{RawFunction, TypedFields};
use crate::stdlib;
use crate::symbols::SymbolFunc;
use crate::types::{Type, VerifiedType};

pub fn is_std_function(func_name: &str) -> bool {
    stdlib::STD_FUNCTIONS.iter().any(|(k, _)| *k == func_name)
}

fn std_function_signatures() -> HashMap<&'static str, (Vec<VerifiedType>, VerifiedType)> {
    // TODO: review return types when void is done!
    HashMap::from(stdlib::STD_FUNCTIONS.map(|(k, v)| (k, v())))
}

pub fn get_std_method(t: &VerifiedType, method_name: &str) -> Result<Box<RawFunction>, String> {
    let mut type_methods = match t {
        Type::Bool => stdlib::STD_BOOL_METHODS.iter(),
        Type::Int => stdlib::STD_INT_METHODS.iter(),
        Type::Float => stdlib::STD_FLOAT_METHODS.iter(),
        Type::String => stdlib::STD_STRING_METHODS.iter(),
        Type::List(_) => stdlib::STD_LIST_METHODS.iter(),
        _ => return Err(format!("Unsupported type for std method: {}", t)),
    };

    let (mut args, return_type) = match type_methods.find(|(k, _)| *k == method_name) {
        Some((_, v)) => v(t),
        None => return Err(format!("Not found method {} for type {}", method_name, t)),
    };
    args.insert(0, t.clone());

    Ok(Box::new(RawFunction {
        name: SymbolFunc::new_std_method(t, method_name),
        return_type,
        args: TypedFields {
            types: args.clone(),
            names: args.iter().enumerate().map(|(i, _)| (i, "".into())).collect(),
        },
        body: vec![],
        locals: vec![],
        short_name: method_name.into(),
        method_of: None,
        is_constructor: false,
        defined_at: ModuleAlias::std(),
    }))
}

pub fn get_std_function_raw(name: &str) -> RawFunction {
    let (args, return_type) = &std_function_signatures()[name];
    RawFunction {
        name: SymbolFunc::new_std_function(name),
        return_type: return_type.clone(),
        args: TypedFields {
            types: args.clone(),
            names: args.iter().enumerate().map(|(i, _)| (i, "".into())).collect(),
        },
        body: vec![],
        locals: vec![],
        short_name: name.into(),
        method_of: None,
        is_constructor: false,
        defined_at: ModuleAlias::std(),
    }
}
