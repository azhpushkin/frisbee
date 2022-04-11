#[test]
pub fn check_constructor() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Person {
            String name;
            Int age;

            fun Person() {}
        }
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_ok());

    let fields = HashMap::from([("name".into(), Type::String), ("age".into(), Type::Int)]);
    let constructor = FunctionSignature {
        rettype: Type::IdentQualified(new_alias("main"), "Person".into()),
        args: vec![],
    };

    let person_signature = ClassSignature {
        module_path_alias: new_alias("main"),
        name: "Person".into(),
        is_active: false,
        fields: fields.clone(),
        methods: HashMap::from([("Person".into(), constructor)]),
    };
    assert_eq!(
        modules::get_typenames_signatures(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "Person"), person_signature)])
    );
}

#[test]
pub fn check_default_constructor() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Person {
            String name;
            Int age;
        }
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_ok());

    let fields = vec![("name".into(), Type::String), ("age".into(), Type::Int)];
    let default_constructor = FunctionSignature {
        rettype: Type::IdentQualified(new_alias("main"), "Person".into()),
        args: fields.clone(),
    };

    let person_signature = ClassSignature {
        module_path_alias: new_alias("main"),
        name: "Person".into(),
        is_active: false,
        fields: fields.into_iter().collect(),
        methods: HashMap::from([("Person".into(), default_constructor)]),
    };
    assert_eq!(
        modules::get_typenames_signatures(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "Person"), person_signature)])
    );
}