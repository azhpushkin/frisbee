use std::collections::HashMap;

use crate::ast::{ModulePath, ModulePathAlias, Type};
use crate::loader::{LoadedFile, WholeProgram};
use crate::semantic_checker::check_and_gather_symbols_mappings;
use crate::test_utils::{new_alias, setup_and_load_program};

use super::super::modules::*;
use super::super::semantic_error::SemanticResult;
use super::super::symbols::*;

fn new_symbol(module: &str, name: &str) -> SymbolOrigin {
    SymbolOrigin { module: new_alias(module), name: name.into() }
}

fn get_file<'a>(wp: &'a WholeProgram, module: &str) -> &'a LoadedFile {
    &wp.files
        .get(&new_alias(module))
        .expect(format!("Module {} not found", module).as_str())
}

fn get_functions_signatures_helper(file: &LoadedFile) -> HashMap<SymbolOrigin, FunctionSignature> {
    let type_map = get_typenames_mapping(file).unwrap();
    get_functions_signatures(file, &type_map).unwrap()
}

fn get_typenames_signatures_helper(file: &LoadedFile) -> HashMap<SymbolOrigin, ClassSignature> {
    let type_map = get_typenames_mapping(file).unwrap();
    get_typenames_signatures(file, &type_map).unwrap()
}

#[test]
pub fn check_import_from_same_module_is_fine() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import Type;

        ===== file: mod.frisbee
        fun Nil somefun() {}
        class Type {}
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_ok());

    let funcs_mapping: SemanticResult<SymbolOriginsMapping> = Ok(HashMap::from([(
        "somefun".into(),
        new_symbol("mod", "somefun"),
    )]));
    let types_mapping: SemanticResult<SymbolOriginsMapping> =
        Ok(HashMap::from([("Type".into(), new_symbol("mod", "Type"))]));
    // Types and functions mappings are the same
    assert_eq!(get_functions_mapping(get_file(&wp, "main")), funcs_mapping);
    assert_eq!(get_functions_mapping(get_file(&wp, "mod")), funcs_mapping);
    assert_eq!(get_typenames_mapping(get_file(&wp, "main")), types_mapping);
    assert_eq!(get_typenames_mapping(get_file(&wp, "mod")), types_mapping);
}

#[test]
pub fn check_import_of_same_function_are_not_allowed() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;
        from mod import somefun;

        
        ===== file: mod.frisbee
        fun Nil somefun() {}
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_import_function_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import somefun;

        fun Nil somefun() {}
        ===== file: mod.frisbee
        fun Bool somefun() {}
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_import_active_type_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from mod import Type;

        active Type {}
        ===== file: mod.frisbee
          // empty file
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_active_and_class_name_collision() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {}
        active Type {}
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_method_name_collisions() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {
            fun Nil hello() {}
            fun Nil hello() {}
        }
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_same_function_names_are_fine() {
    let wp = setup_and_load_program(
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

    assert!(check_and_gather_symbols_mappings(&wp).is_ok());

    assert_eq!(
        get_functions_mapping(get_file(&wp, "main")).unwrap(),
        HashMap::from([
            ("hello".into(), new_symbol("mod", "hello")),
            ("samename".into(), new_symbol("main", "samename")),
        ])
    );
    assert_eq!(
        get_functions_mapping(get_file(&wp, "mod")).unwrap(),
        HashMap::from([
            ("hello".into(), new_symbol("mod", "hello")),
            ("samename".into(), new_symbol("mod", "samename")),
        ])
    );

    let samename_main = FunctionSignature {
        rettype: Type::TypeNil,
        args: vec![(
            "someone".into(),
            Type::TypeIdentQualified(new_alias("mod"), "Person".into()),
        )],
    };
    let samename_mod = FunctionSignature {
        rettype: Type::TypeIdentQualified(new_alias("mod"), "Person".into()),
        args: vec![],
    };
    let hello_mod = FunctionSignature { rettype: Type::TypeNil, args: vec![] };
    assert_eq!(
        get_functions_signatures_helper(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "samename"), samename_main)])
    );
    assert_eq!(
        get_functions_signatures_helper(get_file(&wp, "mod")),
        HashMap::from([
            (new_symbol("mod", "samename"), samename_mod),
            (new_symbol("mod", "hello"), hello_mod),
        ])
    );
}

#[test]
pub fn check_constructor() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Person {
            String name;
            Int age;

            fun Person() {}
        }
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_ok());

    let fields = HashMap::from([("name".into(), Type::TypeString), ("age".into(), Type::TypeInt)]);
    let constructor = FunctionSignature {
        rettype: Type::TypeIdentQualified(new_alias("main"), "Person".into()),
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
        get_typenames_signatures_helper(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "Person"), person_signature)])
    );
}

#[test]
pub fn check_default_constructor() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Person {
            String name;
            Int age;
        }
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_ok());

    let fields = vec![("name".into(), Type::TypeString), ("age".into(), Type::TypeInt)];
    let default_constructor = FunctionSignature {
        rettype: Type::TypeIdentQualified(new_alias("main"), "Person".into()),
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
        get_typenames_signatures_helper(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "Person"), person_signature)])
    );
}

#[test]
pub fn check_self_referrings_for_active_are_allowed() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        active Type { Type type; }
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_ok());

    let default_constructor = FunctionSignature {
        rettype: Type::TypeIdentQualified(new_alias("main"), "Type".into()),
        args: vec![(
            "type".into(),
            Type::TypeIdentQualified(new_alias("main"), "Type".into()),
        )],
    };
    let type_signature = ClassSignature {
        module_path_alias: new_alias("main"),
        name: "Type".into(),
        is_active: true,
        fields: HashMap::from([(
            "type".into(),
            Type::TypeIdentQualified(new_alias("main"), "Type".into()),
        )]),
        methods: HashMap::from([("Type".into(), default_constructor)]),
    };
    assert_eq!(
        get_typenames_signatures_helper(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "Type"), type_signature)])
    );
}

#[test]
pub fn check_no_self_referrings_for_passive() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { Type type; }
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_no_self_referrings_for_tuple() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { (Type, Int) type; }
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_no_self_referrings_in_imports() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_imported_types_are_existing() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import X1;
        ===== file: module.frisbee
        class X {}
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}

#[test]
pub fn check_imported_functions_are_existing() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import func;
        ===== file: module.frisbee
            // empty file
    "#,
    );

    assert!(check_and_gather_symbols_mappings(&wp).is_err());
}
