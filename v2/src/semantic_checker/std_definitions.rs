use crate::ast::*;
use std::collections::HashMap;

fn get_list_methods(inside: &Type) -> ObjectDecl {
    ObjectDecl {
        is_active: false,
        name: "List".into(),
        fields: HashMap::new(),
        methods: HashMap::from([
            (
                "size".into(),
                FunctionDecl {
                    rettype: Type::TypeInt,
                    name: "size".into(),
                    args: HashMap::new(),
                    statements: vec![],
                },
            ),
            (
                "push".into(),
                FunctionDecl {
                    rettype: Type::TypeNil,
                    name: "push".into(),
                    args: HashMap::from([(
                        "item".into(),
                        TypedNamedObject { typename: inside.clone(), name: "item".into() },
                    )]),
                    statements: vec![],
                },
            ),
        ]),
    }
}

pub fn get_std_methods(t: &Type) -> ObjectDecl {
    if matches!(t, Type::TypeList(..)) {
        return get_list_methods(t);
    }
    panic!("Not implemented");
}
