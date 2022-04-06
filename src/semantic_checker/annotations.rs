use crate::ast::*;
use crate::semantic_checker::semantic_error::SemanticResult;

use super::semantic_error::sem_err;
use super::symbols::SymbolOriginsMapping;

// pub fn check_and_annotate_ast_in_place(
//     file_ast: &mut FileAst,
//     file_module: &ModulePathAlias,
//     info: &GlobalSymbolsInfo,
// ) -> SemanticResult<()> {
//     let types_per_file = &info.symbols_per_file[&file_module].typenames;

// }

pub fn annotate_class_decl(
    class_decl: &mut ClassDecl,
    typenames_origins: &SymbolOriginsMapping,
) -> SemanticResult<()> {
    for field in class_decl.fields.iter_mut() {
        field.typename = annotate_type(&field.typename, typenames_origins)?;
    }

    for method in class_decl.methods.iter_mut() {
        annotate_function_decl(method, typenames_origins)?;
    }
    // If no constructor is mentioned - then add default one for typecheck
    if !class_decl.methods.iter().any(|m| class_decl.name == m.name) {
        class_decl.methods.push(FunctionDecl {
            name: class_decl.name.clone(),
            rettype: annotate_type(&Type::TypeIdent(class_decl.name.clone()), typenames_origins)?,
            args: class_decl.fields.clone(),
            statements: vec![], // TODO: fill will with required AST
        });
    }

    Ok(())
}

pub fn annotate_function_decl(
    func_decl: &mut FunctionDecl,
    typenames_origins: &SymbolOriginsMapping,
) -> SemanticResult<()> {
    func_decl.rettype = annotate_type(&func_decl.rettype, typenames_origins)?;
    for param in func_decl.args.iter_mut() {
        param.typename = annotate_type(&param.typename, typenames_origins)?;
    }
    Ok(())
}

pub fn annotate_type(t: &Type, typenames_mapping: &SymbolOriginsMapping) -> SemanticResult<Type> {
    let new_t = match t {
        Type::TypeInt => Type::TypeInt,
        Type::TypeFloat => Type::TypeFloat,
        Type::TypeNil => Type::TypeNil,
        Type::TypeBool => Type::TypeBool,
        Type::TypeString => Type::TypeString,

        Type::TypeList(t) => {
            Type::TypeList(Box::new(annotate_type(t.as_ref(), typenames_mapping)?))
        }
        Type::TypeMaybe(t) => {
            Type::TypeMaybe(Box::new(annotate_type(t.as_ref(), typenames_mapping)?))
        }
        Type::TypeTuple(ts) => {
            let ts_annotated: SemanticResult<Vec<Type>> =
                ts.iter().map(|t| annotate_type(t, typenames_mapping)).collect();
            Type::TypeTuple(ts_annotated?)
        }

        Type::TypeIdent(s) => {
            let symbol_origin = typenames_mapping.get(s);
            if let Some(symbol_origin) = symbol_origin {
                Type::TypeIdentQualified(symbol_origin.module.clone(), symbol_origin.name.clone())
            } else {
                return sem_err!("Unknown type {}", s);
            }
        }
        Type::TypeIdentQualified(..) => t.clone(),
        Type::TypeAnonymous => panic!("Did not expected {:?}", t),
    };
    Ok(new_t)
}
