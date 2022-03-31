use crate::ast::{ModulePath, ModulePathAlias, Type};
use crate::loader::{LoadedFile, WholeProgram};
use crate::semantic_checker::{modules::*, perform_checks, semantic_error::SemanticResult};
use crate::test_utils::setup_and_load_program;

use std::collections::HashMap;

fn new_obj_path(module: &str, name: &str) -> ObjectPath {
    (new_alias(module), name.into())
}

fn new_alias(module: &str) -> ModulePathAlias {
    // NOTE: this does not work for module.submodule right now
    ModulePath(vec![module.to_string()]).alias()
}

fn get_file<'a>(wp: &'a WholeProgram, module: &str) -> &'a LoadedFile {
    &wp.files
        .get(&new_alias(module))
        .expect(format!("Module {} not found", module).as_str())
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

    assert!(perform_checks(&wp).is_ok());

    let funcs_mapping: SemanticResult<FileObjectsMapping> = Ok(HashMap::from([(
        "somefun".into(),
        new_obj_path("mod", "somefun"),
    )]));
    let types_mapping: SemanticResult<FileObjectsMapping> = Ok(HashMap::from([(
        "Type".into(),
        new_obj_path("mod", "Type"),
    )]));
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

    assert!(perform_checks(&wp).is_err());
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

    assert!(perform_checks(&wp).is_err());
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

    assert!(perform_checks(&wp).is_err());
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

    assert!(perform_checks(&wp).is_err());
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

    assert!(perform_checks(&wp).is_err());
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

    assert!(perform_checks(&wp).is_ok());

    assert_eq!(
        get_functions_mapping(get_file(&wp, "main")).unwrap(),
        HashMap::from([
            ("hello".into(), new_obj_path("mod", "hello")),
            ("samename".into(), new_obj_path("main", "samename")),
        ])
    );
    assert_eq!(
        get_functions_mapping(get_file(&wp, "mod")).unwrap(),
        HashMap::from([
            ("hello".into(), new_obj_path("mod", "hello")),
            ("samename".into(), new_obj_path("mod", "samename")),
        ])
    );

    let samename_main = FunctionSignature {
        rettype: Type::TypeNil,
        args: HashMap::from([(
            "someone".into(),
            Type::TypeIdentQualified(new_alias("mod"), "Person".into()),
        )]),
    };
    let samename_mod = FunctionSignature {
        rettype: Type::TypeIdentQualified(new_alias("mod"), "Person".into()),
        args: HashMap::new(),
    };
    let hello = FunctionSignature { rettype: Type::TypeNil, args: HashMap::new() };
    // assert_eq!(
    //     get_functions_signatures(
    //         get_file(&wp, "main"),
    //         get_typenames_mapping(get_file(&wp, "main")).unwrap()
    //     ).unwrap(),
    //     HashMap::from(vec![
    //         (new_obj_path())
    //     ])
    // )
}

#[test]
pub fn check_self_referrings_for_active_are_allowed() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        active Type { Type type; }
    "#,
    );

    assert!(perform_checks(&wp).is_ok());
}

#[test]
pub fn check_no_self_referrings_for_passive() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { Type type; }
    "#,
    );

    assert!(perform_checks(&wp).is_err());
}

#[test]
pub fn check_no_self_referrings_for_tuple() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        class Type { (Type, Int) type; }
    "#,
    );

    assert!(perform_checks(&wp).is_err());
}

#[test]
pub fn check_no_self_referrings_in_imports() {
    let wp = setup_and_load_program(
        r#"
        ===== file: main.frisbee
        from main import Type;
    "#,
    );

    assert!(perform_checks(&wp).is_err());
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

    assert!(perform_checks(&wp).is_err());
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

    assert!(perform_checks(&wp).is_err());
}
