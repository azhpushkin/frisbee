use std::collections::HashMap;

use crate::ast::*;

use super::operators::{are_types_same_or_maybe, calculate_binaryop_type, calculate_unaryop_type};
use super::semantic_error::{sem_err, SemanticResult};
use super::std_definitions::get_std_method;
use super::symbols::*;

pub struct ExprTypeChecker<'a> {
    symbols_info: &'a GlobalSymbolsInfo,
    file_name: ModulePathAlias,
    scope: Option<String>,
    variables_types: HashMap<String, Type>,
}

impl<'a> ExprTypeChecker<'a> {
    pub fn new(
        symbols_info: &'a GlobalSymbolsInfo,
        file_name: ModulePathAlias,
        scope: Option<String>,
    ) -> ExprTypeChecker<'a> {
        ExprTypeChecker { symbols_info, file_name, scope, variables_types: HashMap::new() }
    }

    pub fn add_variable(&mut self, name: String, t: Type) -> SemanticResult<()> {
        if self.variables_types.contains_key(&name) {
            return sem_err!("Variable {} already declared", name);
        }
        self.variables_types.insert(name, t);
        Ok(())
    }

    fn err_prefix(&self) -> String {
        format!("In file {}: ", self.file_name.0)
    }

    fn calculate_vec(&self, items: &mut Vec<Expr>) -> SemanticResult<Vec<Type>> {
        let calculated_items = items.iter_mut().map(|item| self.calculate_and_annotate(item));
        let unwrapped_items: SemanticResult<Vec<Type>> = calculated_items.collect();
        Ok(unwrapped_items?)
    }

    fn get_symbols_per_file(&self) -> &SymbolOriginsPerFile {
        self.symbols_info.symbols_per_file.get(&self.file_name).unwrap()
    }

    fn get_class_signature(&self, typename: &String) -> SemanticResult<&ClassSignature> {
        let class_origin = match self.get_symbols_per_file().typenames.get(typename) {
            Some(origin) => origin,
            None => return sem_err!("{} Type {} not found", self.err_prefix(), typename),
        };
        // Unwrap here as during global_signatures creation we checked that the type exists
        Ok(self
            .symbols_info
            .global_signatures
            .typenames
            .get(class_origin)
            .unwrap())
    }

    fn get_function_file(&self, function_name: &String) -> ModulePathAlias {
        self.get_symbols_per_file()
            .functions
            .get(function_name)
            .unwrap()
            .module
            .clone()
    }

    fn get_function_signature(&self, function_name: &String) -> SemanticResult<&FunctionSignature> {
        let function_origin = match self.get_symbols_per_file().functions.get(function_name) {
            Some(origin) => origin,
            None => return sem_err!("{} Function {} not found", self.err_prefix(), function_name),
        };
        // Unwrap here as during global_signatures creation we checked that the function exists
        Ok(self
            .symbols_info
            .global_signatures
            .functions
            .get(function_origin)
            .unwrap())
    }

    fn get_type_method(
        &self,
        alias: &ModulePathAlias,
        name: &String,
        method: &String,
    ) -> SemanticResult<&FunctionSignature> {
        let symbol_origin = SymbolOrigin { module: alias.clone(), name: name.clone() };
        let class_signature = self
            .symbols_info
            .global_signatures
            .typenames
            .get(&symbol_origin)
            .unwrap();
        self.get_from_class_signature(&class_signature.methods, method)
    }

    fn get_type_field(
        &self,
        alias: &ModulePathAlias,
        name: &String,
        field: &String,
    ) -> SemanticResult<&Type> {
        let symbol_origin = SymbolOrigin { module: alias.clone(), name: name.clone() };
        let class_signature = self
            .symbols_info
            .global_signatures
            .typenames
            .get(&symbol_origin)
            .unwrap();
        self.get_from_class_signature(&class_signature.fields, field)
    }

    fn get_from_class_signature<'q, T>(
        &self,
        mapping: &'q HashMap<String, T>,
        name: &String,
    ) -> SemanticResult<&'q T> {
        match mapping.get(name) {
            Some(value) => Ok(value),
            None => sem_err!("{} {} not found", self.err_prefix(), name),
        }
    }

    pub fn calculate_and_annotate(&self, expr: &mut Expr) -> SemanticResult<Type> {
        let calculated_type = match expr {
            // Primitive types, that map to basic types
            Expr::Int(_) => Type::Int,
            Expr::String(_) => Type::String,
            Expr::Bool(_) => Type::Bool,
            Expr::Nil => Type::Nil,
            Expr::Float(_) => Type::Float,

            // Simple lookup is enough for this
            Expr::Identifier(identifier) => match self.variables_types.get(identifier) {
                Some(t) => t.clone(),
                None => return sem_err!("{} unknown variable {}", self.err_prefix(), identifier),
            },

            Expr::TupleValue(items) => Type::Tuple(self.calculate_vec(items)?),
            Expr::ListValue(items) => {
                if items.len() == 0 {
                    // TODO: tests for anonymous type (in let and in methods)
                    Type::List(Box::new(Type::Anonymous))
                } else {
                    let calculated_items = self.calculate_vec(items)?;
                    // Check for maybe types required, but this is not for now
                    if calculated_items.windows(2).any(|pair| pair[0] != pair[1]) {
                        return sem_err!(
                            "{} list items have different types: {:?}",
                            self.err_prefix(),
                            calculated_items
                        );
                    }
                    Type::List(Box::new(calculated_items[0].clone()))
                }
            }

            Expr::UnaryOp { op, operand } => {
                calculate_unaryop_type(op, &self.calculate_and_annotate(operand)?)?
            }
            Expr::BinOp { left, right, op } => calculate_binaryop_type(
                op,
                &self.calculate_and_annotate(left)?,
                &self.calculate_and_annotate(right)?,
            )?,

            Expr::ListAccess { list, index } => self.calculate_access_by_index(list, index)?,

            Expr::NewClassInstance { typename, args } => {
                let class_signature = self.get_class_signature(typename)?;
                if class_signature.is_active {
                    return sem_err!("{} You must call spawn for {}", self.err_prefix(), typename);
                }

                let constuctor =
                    class_signature.methods.get(typename).expect("Constructor not found");
                self.check_function_call(constuctor, args)?
            }
            Expr::SpawnActive { typename, args } => {
                let class_signature = self.get_class_signature(typename)?;

                if !class_signature.is_active {
                    return sem_err!("{} Cant spawn passive {}!", self.err_prefix(), typename);
                }
                let constuctor =
                    class_signature.methods.get(typename).expect("Constructor not found");
                self.check_function_call(constuctor, args)?
            }
            Expr::FunctionCall { function, args } => {
                let function_signature = self.get_function_signature(function)?;
                let rettype = self.check_function_call(function_signature, args)?;
                *expr = Expr::FunctionCallQualified {
                    module: self.get_function_file(function),
                    function: std::mem::take(function),
                    args: std::mem::take(args),
                };
                rettype
            }

            Expr::MethodCall { object, method, args } => {
                // TODO: implement something for built-in types
                let obj_type = self.calculate_and_annotate(object)?;
                match &obj_type {
                    Type::IdentQualified(alias, name) => {
                        let method = self.get_type_method(alias, name, method)?;
                        self.check_function_call(method, args)?
                    }
                    Type::Ident(..) => panic!("TypeIdent should not be present here!"),
                    Type::Maybe(_) => panic!("Not implemented for maybe yet!"), // implement ?.
                    t => {
                        let method = get_std_method(t, method)?;
                        self.check_function_call(&method, args)?
                    }
                }
            }

            Expr::FieldAccess { object, field } => {
                // TODO: implement something for built-in types
                let obj_type = self.calculate_and_annotate(object)?;
                match &obj_type {
                    Type::IdentQualified(alias, name) => {
                        let field_type = self.get_type_field(alias, name, field)?;
                        field_type.clone()
                    }
                    Type::Ident(..) => panic!("TypeIdent should not be present here!"),
                    _ => {
                        return sem_err!(
                            "Error at {:?} - type {:?} has no fields",
                            object,
                            obj_type
                        )
                    }
                }
            }
            Expr::OwnMethodCall { .. } | Expr::OwnFieldAccess { .. } if self.scope.is_none() => {
                return sem_err!("{} Using @ is not allowed in functions", self.err_prefix())
            }
            Expr::OwnMethodCall { method, args } => {
                let method_signature =
                    self.get_type_method(&self.file_name, self.scope.as_ref().unwrap(), method)?;
                self.check_function_call(method_signature, args)?
            }
            Expr::OwnFieldAccess { field } => {
                let field_type =
                    self.get_type_field(&self.file_name, &self.scope.as_ref().unwrap(), field)?;
                field_type.clone()
            }
            Expr::This => match &self.scope {
                None => return sem_err!("Using 'this' in the functions is not allowed!"),
                Some(_) => {
                    Type::IdentQualified(self.file_name.clone(), self.scope.clone().unwrap())
                }
            },
            _ => todo!("asd"),
        };

        let mut temp_box = Expr::Nil;
        std::mem::swap(expr, &mut temp_box);

        // Now temp box has the original expr, so we can move it to wrapper one
        let mut new_expr =
            Expr::TypedExpr { expr: Box::new(temp_box), typename: calculated_type.clone() };
        std::mem::swap(expr, &mut new_expr);

        // Now boxed Expr::Nil is in the new_expr variable and expr has new correct value

        Ok(calculated_type)
    }

    fn check_function_call(
        &self,
        function: &FunctionSignature,
        args: &mut Vec<Expr>,
    ) -> SemanticResult<Type> {
        if function.args.len() != args.len() {
            return sem_err!(
                "{} Wrong amount of arguments at {:?}, expected {}",
                self.err_prefix(),
                args,
                function.args.len()
            );
        }

        for (expected_arg, arg_expr) in function.args.iter().zip(args.iter_mut()) {
            let expr_type = self.calculate_and_annotate(arg_expr)?;
            // TODO: this is wrong check of type correctness, but it works for now
            if !are_types_same_or_maybe(&expected_arg.1, &expr_type) {
                return sem_err!(
                    "Wrong type for argument {}, expected {:?}, got {:?} ({:?})",
                    expected_arg.0,
                    expected_arg.1,
                    expr_type,
                    arg_expr
                );
            }
        }

        Ok(function.rettype.clone())
    }

    fn calculate_access_by_index(
        &self,
        list: &mut Box<Expr>,
        index: &mut Box<Expr>,
    ) -> Result<Type, String> {
        let list_type = self.calculate_and_annotate(list.as_mut())?;
        match list_type {
            Type::List(item) => match self.calculate_and_annotate(index.as_mut())? {
                Type::Int => Ok(item.as_ref().clone()),
                t => sem_err!("List index must be int, but got {:?} in {:?}", t, index),
            },
            Type::Tuple(items) => match index.as_ref() {
                Expr::Int(i) => {
                    let item = items.get(*i as usize);
                    if item.is_some() {
                        Ok(item.unwrap().clone())
                    } else {
                        sem_err!(
                            "{} Out of bounds index in {:?}",
                            self.err_prefix(),
                            list.as_ref()
                        )
                    }
                }
                _ => sem_err!("Not int for tuple access in {:?}", list.as_ref()),
            },
            _ => sem_err!(
                "Expected tuple or list for index access, got {:?} in {:?}",
                list_type,
                list.as_ref()
            ),
        }
    }
}
