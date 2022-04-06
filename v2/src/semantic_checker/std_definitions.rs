use std::collections::HashMap;

use crate::ast::Type;
use crate::semantic_checker::semantic_error::sem_err;

use super::semantic_error::SemanticResult;
use super::symbols::FunctionSignature;

fn get_list_methods(inside: &Type) -> HashMap<String, FunctionSignature> {
    HashMap::from([
        (
            "size".into(),
            FunctionSignature { rettype: Type::TypeInt, args: vec![] },
        ),
        (
            "push".into(),
            FunctionSignature {
                rettype: Type::TypeNil,
                args: vec![("item".into(), inside.clone())],
            },
        ),
    ])
}


fn get_int_methods() -> HashMap<String, FunctionSignature> {
    HashMap::new()
}

fn get_float_methods() -> HashMap<String, FunctionSignature> {
    HashMap::new()
}

fn get_nil_methods() -> HashMap<String, FunctionSignature> {
    HashMap::new()
}

fn get_bool_methods() -> HashMap<String, FunctionSignature> {
    HashMap::new()
}

fn get_string_methods() -> HashMap<String, FunctionSignature> {
    HashMap::new()
}


fn get_std_methods(t: &Type) -> HashMap<String, FunctionSignature> {
    match t {
        Type::TypeInt => get_int_methods(),
        Type::TypeFloat => get_float_methods(),
        Type::TypeNil => get_nil_methods(),
        Type::TypeBool => get_bool_methods(),
        Type::TypeString => get_string_methods(),
        Type::TypeList(inside) => get_list_methods(inside.as_ref()),
        _ => panic!("Not implemented std for {:?}", t)
    }
}

pub fn get_std_method(t: &Type, method: &String) -> SemanticResult<FunctionSignature> {
    let mut methods = get_std_methods(t);
    let signature = methods.remove(method);
    if signature.is_some() {
        Ok(signature.unwrap())
    } else {
        sem_err!("Cant find method {} for type {:?}", method, t)
    }
}
