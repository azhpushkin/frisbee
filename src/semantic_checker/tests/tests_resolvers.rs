use crate::semantic_checker::resolvers::NameResolver;
use crate::test_utils::setup_and_load_program;

#[test]
pub fn check_import_from_same_module_is_fine() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import Type;

        ===== file: mod.frisbee
        fun Nil somefun() {}
        class Type {}
    "#,
    );

    NameResolver::create(&mut wp);
    // assert!(info.is_ok(), "{:?}", info.unwrap_err());

    // let funcs_mapping: SemanticResult<SymbolOriginsMapping> =
    //     Ok([("somefun".into(), new_symbol("mod", "somefun"))].into());
    // let types_mapping: SemanticResult<SymbolOriginsMapping> =
    //     Ok([("Type".into(), new_symbol("mod", "Type"))].into());

    // // Types and functions mappings are the same
    // let main_file = get_file(&wp, "main");
    // let mod_file = get_file(&wp, "mod");
    // assert_eq!(modules::get_functions_origins(main_file), funcs_mapping);
    // assert_eq!(modules::get_functions_origins(mod_file), funcs_mapping);

    // assert_eq!(modules::get_typenames_origins(main_file), types_mapping);
    // assert_eq!(modules::get_typenames_origins(mod_file), types_mapping);
}

#[test]
#[should_panic]
pub fn check_import_of_same_function_are_not_allowed() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import somefun;

        
        ===== file: mod.frisbee
        fun Nil somefun() {}
    "#,
    );

    NameResolver::create(&mut wp);
}

#[test]
#[should_panic]
pub fn check_import_function_name_collision() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;

        fun Nil somefun() {}
        ===== file: mod.frisbee
        fun Bool somefun() {}
    "#,
    );

    NameResolver::create(&mut wp);
}

#[test]
#[should_panic]
pub fn check_import_active_type_name_collision() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import Type;

        active Type {}
        ===== file: mod.frisbee
          // empty file
    "#,
    );

    NameResolver::create(&mut wp);
}

#[test]
#[should_panic]
pub fn check_active_and_class_name_collision() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {}
        active Type {}
    "#,
    );

    NameResolver::create(&mut wp);
}

#[test]
#[should_panic]
pub fn check_method_name_collisions() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {
            fun Nil hello() {}
            fun Nil hello() {}
        }
    "#,
    );

    NameResolver::create(&mut wp);
}

#[test]
pub fn check_same_function_names_are_fine() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import hello, Person;

        fun Nil samename(Person someone) {}
        ===== file: mod.frisbee
        fun Person samename() {}
        fun Nil hello() {}

        class Person {}
    "#,
    );
    NameResolver::create(&mut wp);

    // assert!(check_and_annotate_symbols(&mut wp).is_ok());

    // assert_eq!(
    //     modules::get_functions_origins(get_file(&wp, "main")).unwrap(),
    //     HashMap::from([
    //         ("hello".into(), new_symbol("mod", "hello")),
    //         ("samename".into(), new_symbol("main", "samename")),
    //     ])
    // );
    // assert_eq!(
    //     modules::get_functions_origins(get_file(&wp, "mod")).unwrap(),
    //     HashMap::from([
    //         ("hello".into(), new_symbol("mod", "hello")),
    //         ("samename".into(), new_symbol("mod", "samename")),
    //     ])
    // );

    // let samename_main = FunctionSignature {
    //     rettype: Type::Nil,
    //     args: vec![(
    //         "someone".into(),
    //         Type::IdentQualified(new_alias("mod"), "Person".into()),
    //     )],
    // };
    // let samename_mod = FunctionSignature {
    //     rettype: Type::IdentQualified(new_alias("mod"), "Person".into()),
    //     args: vec![],
    // };
    // let hello_mod = FunctionSignature { rettype: Type::Nil, args: vec![] };
    // assert_eq!(
    //     modules::get_functions_signatures(get_file(&wp, "main")),
    //     HashMap::from([(new_symbol("main", "samename"), samename_main)])
    // );
    // assert_eq!(
    //     modules::get_functions_signatures(get_file(&wp, "mod")),
    //     HashMap::from([
    //         (new_symbol("mod", "samename"), samename_mod),
    //         (new_symbol("mod", "hello"), hello_mod),
    //     ])
    // );
}

#[test]
#[should_panic]
pub fn check_no_self_referrings_in_imports() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    NameResolver::create(&mut wp);
}

#[test]
#[should_panic]
pub fn check_imported_types_are_existing() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import X1;
        ===== file: module.frisbee
        class X {}
    "#,
    );

    NameResolver::create(&mut wp);
}

#[test]
#[should_panic]
pub fn check_imported_functions_are_existing() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import func;
        ===== file: module.frisbee
            // empty file
    "#,
    );

    NameResolver::create(&mut wp);
}
