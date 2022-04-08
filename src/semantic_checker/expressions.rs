use std::collections::HashMap;

use crate::ast::*;

use super::operators::{are_types_same_or_maybe, calculate_binaryop_type, calculate_unaryop_type};
use super::semantic_error::{sem_err, SemanticResult};
use super::std_definitions::{get_std_method};
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
        let prev = self.variables_types.insert(name, t);
        match prev {
            Some(_) => sem_err!("Variable {} already declared", name),
            None => Ok(()),
        }
    }

    pub fn reset_variables(&mut self) {
        self.variables_types.clear();
    }

    fn err_prefix(&self) -> String {
        format!("In file {}: ", self.file_name.0)
    }

    fn calculate_vec(&self, items: &Vec<ExprRaw>) -> SemanticResult<Vec<Type>> {
        let calculated_items = items.iter().map(|item| self.calculate(item));
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
        Ok(self.symbols_info.global_signatures.typenames.get(class_origin).unwrap())
    }

    fn get_function_signature(&self, function_name: &String) -> SemanticResult<&FunctionSignature> {
        let function_origin = match self.get_symbols_per_file().functions.get(function_name) {
            Some(origin) => origin,
            None => return sem_err!("{} Function {} not found", self.err_prefix(), function_name),
        };
        // Unwrap here as during global_signatures creation we checked that the function exists
        Ok(self.symbols_info.global_signatures.functions.get(function_origin).unwrap())
    }

    fn get_type_method(
        &self,
        alias: &ModulePathAlias,
        name: &String,
        method: &String,
    ) -> SemanticResult<&FunctionSignature> {
        let symbol_origin = SymbolOrigin { module: alias.clone(), name: name.clone() };
        let class_signature =
            self.symbols_info.global_signatures.typenames.get(&symbol_origin).unwrap();
        self.get_from_class_signature(&class_signature.methods, method)
    }

    fn get_type_field(
        &self,
        alias: &ModulePathAlias,
        name: &String,
        field: &String,
    ) -> SemanticResult<&Type> {
        let symbol_origin = SymbolOrigin { module: alias.clone(), name: name.clone() };
        let class_signature =
            self.symbols_info.global_signatures.typenames.get(&symbol_origin).unwrap();
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

    pub fn calculate(&self, expr: &ExprRaw) -> SemanticResult<Type> {
        match expr {
            // Primitive types, that map to basic types
            ExprRaw::Int(_) => Ok(Type::TypeInt),
            ExprRaw::String(_) => Ok(Type::TypeString),
            ExprRaw::Bool(_) => Ok(Type::TypeBool),
            ExprRaw::Nil => Ok(Type::TypeNil),
            ExprRaw::Float(_) => Ok(Type::TypeFloat),

            // Simple lookup is enough for this
            ExprRaw::Identifier(identifier) => match self.variables_types.get(identifier) {
                Some(t) => Ok(t.clone()),
                None => sem_err!("{} unknown variable {}", self.err_prefix(), identifier),
            },

            ExprRaw::TupleValue(items) => Ok(Type::TypeTuple(self.calculate_vec(items)?)),
            ExprRaw::ListValue(items) => {
                if items.len() == 0 {
                    // TODO: tests for anonymous type (in let and in methods)
                    return Ok(Type::TypeList(Box::new(Type::TypeAnonymous)));
                }
                let calculated_items = self.calculate_vec(items)?;
                // Check for maybe types required, but this is not for now
                if calculated_items.windows(2).any(|pair| pair[0] != pair[1]) {
                    return sem_err!(
                        "{} list items have different types: {:?}",
                        self.err_prefix(),
                        calculated_items
                    );
                }
                Ok(Type::TypeList(Box::new(calculated_items[0].clone())))
            }

            ExprRaw::UnaryOp { op, operand } => {
                calculate_unaryop_type(op, &self.calculate(operand)?)
            }
            ExprRaw::BinOp { left, right, op } => {
                calculate_binaryop_type(op, &self.calculate(left)?, &self.calculate(right)?)
            }

            ExprRaw::ListAccess { list, index } => self.calculate_access_by_index(list, index),

            ExprRaw::NewClassInstance { typename, args }
            | ExprRaw::SpawnActive { typename, args } => {
                let class_signature = self.get_class_signature(typename)?;

                if matches!(expr, ExprRaw::NewClassInstance { .. }) && class_signature.is_active {
                    return sem_err!("{} You must call spawn for {}", self.err_prefix(), typename);
                } else if matches!(expr, ExprRaw::SpawnActive { .. }) && !class_signature.is_active
                {
                    return sem_err!("{} Cant spawn passive {}!", self.err_prefix(), typename);
                }
                let constuctor =
                    class_signature.methods.get(typename).expect("Constructor not found");
                return self.check_function_call(constuctor, args);
            }
            ExprRaw::FunctionCall { function, args } => {
                let function_signature = self.get_function_signature(function)?;
                return self.check_function_call(function_signature, args);
            }

            ExprRaw::MethodCall { object, method, args } => {
                // TODO: implement something for built-in types
                let obj_type = self.calculate(object.as_ref())?;
                match &obj_type {
                    Type::TypeIdentQualified(alias, name) => {
                        let method = self.get_type_method(alias, name, method)?;
                        return self.check_function_call(method, args);
                    }
                    Type::TypeIdent(..) => panic!("TypeIdent should not be present here!"),
                    Type::TypeMaybe(_) => panic!("Not implemented for maybe yet!"), // implement ?.
                    t => {
                        let method = get_std_method(t, method)?;
                        return self.check_function_call(&method, args);
                    }
                }
            }

            ExprRaw::FieldAccess { object, field } => {
                // TODO: implement something for built-in types
                let obj_type = self.calculate(object.as_ref())?;
                match &obj_type {
                    Type::TypeIdentQualified(alias, name) => {
                        let field_type = self.get_type_field(alias, name, field)?;
                        Ok(field_type.clone())
                    }
                    Type::TypeIdent(..) => panic!("TypeIdent should not be present here!"),
                    _ => sem_err!("Error at {:?} - type {:?} has no fields", object, obj_type),
                }
            }
            ExprRaw::OwnMethodCall { .. } | ExprRaw::OwnFieldAccess { .. }
                if self.scope.is_none() =>
            {
                sem_err!("{} Using @ is not allowed in functions", self.err_prefix())
            }
            ExprRaw::OwnMethodCall { method, args } => {
                let method_signature =
                    self.get_type_method(&self.file_name, self.scope.as_ref().unwrap(), method)?;
                return self.check_function_call(method_signature, args);
            }
            ExprRaw::OwnFieldAccess { field } => {
                let field_type =
                    self.get_type_field(&self.file_name, &self.scope.as_ref().unwrap(), field)?;
                Ok(field_type.clone())
            }
            ExprRaw::This => match &self.scope {
                None => Err("Using 'this' in the functions is not allowed!".into()),
                Some(o) => Ok(Type::TypeIdentQualified(
                    self.file_name.clone(),
                    self.scope.clone().unwrap(),
                )),
            },
            _ => todo!("asd"),
        }
    }

    fn check_function_call(
        &self,
        function: &FunctionSignature,
        args: &Vec<ExprRaw>,
    ) -> SemanticResult<Type> {
        if function.args.len() != args.len() {
            return sem_err!(
                "{} Wrong amount of arguments at {:?}, expected {}",
                self.err_prefix(),
                args,
                function.args.len()
            );
        }

        for (expected_arg, arg_expr) in function.args.iter().zip(args.iter()) {
            let expr_type = self.calculate(arg_expr)?;
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
        list: &Box<ExprRaw>,
        index: &Box<ExprRaw>,
    ) -> Result<Type, String> {
        let list_type = self.calculate(list.as_ref())?;
        match list_type {
            Type::TypeList(item) => match self.calculate(index)? {
                Type::TypeInt => Ok(item.as_ref().clone()),
                t => sem_err!("List index must be int, but got {:?} in {:?}", t, index),
            },
            Type::TypeTuple(items) => match index.as_ref() {
                ExprRaw::Int(i) => {
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
