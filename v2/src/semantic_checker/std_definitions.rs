use super::modules::FunctionSignature;
use super::semantic_error::SemanticResult;
use crate::ast::Type;
use crate::semantic_checker::semantic_error::sem_err;
use std::collections::HashMap;

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

pub fn get_std_methods(t: &Type) -> HashMap<String, FunctionSignature> {
    if matches!(t, Type::TypeList(..)) {
        return get_list_methods(t);
    }
    panic!("Not implemented std for {:?}", t);
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
