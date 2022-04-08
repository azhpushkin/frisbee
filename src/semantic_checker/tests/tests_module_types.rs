use std::collections::HashMap;

use crate::ast::{ModulePath, ModulePathAlias, Type};
use crate::loader::{LoadedFile, WholeProgram};
use crate::semantic_checker::check_and_annotate_symbols;
use crate::test_utils::{new_alias, setup_and_load_program};

use super::super::modules;
use super::super::semantic_error::SemanticResult;
use super::super::symbols::*;

fn new_symbol(module: &str, name: &str) -> SymbolOrigin {
    SymbolOrigin { module: new_alias(module), name: name.into() }
}

fn get_file<'a>(wp: &'a WholeProgram, module: &str) -> &'a LoadedFile {
    &wp.files.get(&new_alias(module)).expect(format!("Module {} not found", module).as_str())
}

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

    let info = check_and_annotate_symbols(&mut wp);
    assert!(info.is_ok(), "{:?}", info.unwrap_err());

    let funcs_mapping: SemanticResult<SymbolOriginsMapping> =
        Ok([("somefun".into(), new_symbol("mod", "somefun"))].into());
    let types_mapping: SemanticResult<SymbolOriginsMapping> =
        Ok([("Type".into(), new_symbol("mod", "Type"))].into());

    // Types and functions mappings are the same
    let main_file = get_file(&wp, "main");
    let mod_file = get_file(&wp, "mod");
    assert_eq!(modules::get_functions_origins(main_file), funcs_mapping);
    assert_eq!(modules::get_functions_origins(mod_file), funcs_mapping);

    assert_eq!(modules::get_typenames_origins(main_file), types_mapping);
    assert_eq!(modules::get_typenames_origins(mod_file), types_mapping);
}

#[test]
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

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
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

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
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

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
pub fn check_active_and_class_name_collision() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type {}
        active Type {}
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
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

    assert!(check_and_annotate_symbols(&mut wp).is_err());
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

    assert!(check_and_annotate_symbols(&mut wp).is_ok());

    assert_eq!(
        modules::get_functions_origins(get_file(&wp, "main")).unwrap(),
        HashMap::from([
            ("hello".into(), new_symbol("mod", "hello")),
            ("samename".into(), new_symbol("main", "samename")),
        ])
    );
    assert_eq!(
        modules::get_functions_origins(get_file(&wp, "mod")).unwrap(),
        HashMap::from([
            ("hello".into(), new_symbol("mod", "hello")),
            ("samename".into(), new_symbol("mod", "samename")),
        ])
    );

    let samename_main = FunctionSignature {
        rettype: Type::Nil,
        args: vec![(
            "someone".into(),
            Type::IdentQualified(new_alias("mod"), "Person".into()),
        )],
    };
    let samename_mod = FunctionSignature {
        rettype: Type::IdentQualified(new_alias("mod"), "Person".into()),
        args: vec![],
    };
    let hello_mod = FunctionSignature { rettype: Type::Nil, args: vec![] };
    assert_eq!(
        modules::get_functions_signatures(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "samename"), samename_main)])
    );
    assert_eq!(
        modules::get_functions_signatures(get_file(&wp, "mod")),
        HashMap::from([
            (new_symbol("mod", "samename"), samename_mod),
            (new_symbol("mod", "hello"), hello_mod),
        ])
    );
}

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

#[test]
pub fn check_self_referrings_for_active_are_allowed() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        active Type { Type type; }
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_ok());

    let default_constructor = FunctionSignature {
        rettype: Type::IdentQualified(new_alias("main"), "Type".into()),
        args: vec![(
            "type".into(),
            Type::IdentQualified(new_alias("main"), "Type".into()),
        )],
    };
    let type_signature = ClassSignature {
        module_path_alias: new_alias("main"),
        name: "Type".into(),
        is_active: true,
        fields: HashMap::from([(
            "type".into(),
            Type::IdentQualified(new_alias("main"), "Type".into()),
        )]),
        methods: HashMap::from([("Type".into(), default_constructor)]),
    };
    assert_eq!(
        modules::get_typenames_signatures(get_file(&wp, "main")),
        HashMap::from([(new_symbol("main", "Type"), type_signature)])
    );
}

#[test]
pub fn check_no_self_referrings_for_passive() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { Type type; }
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
pub fn check_no_self_referrings_for_tuple() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { (Type, Int) type; }
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
pub fn check_no_self_referrings_in_imports() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
pub fn check_imported_types_are_existing() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import X1;
        ===== file: module.frisbee
        class X {}
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}

#[test]
pub fn check_imported_functions_are_existing() {
    let mut wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from module import func;
        ===== file: module.frisbee
            // empty file
    "#,
    );

    assert!(check_and_annotate_symbols(&mut wp).is_err());
}
