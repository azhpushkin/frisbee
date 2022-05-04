use crate::ast::parsed::*;
use crate::ast::verified::{RawFunction, RawOperator, VExpr, VExprTyped, VStatement};
use crate::symbols::SymbolFunc;
use crate::types::{verify_parsed_type, ParsedType, Type, VerifiedType};

use super::aggregate::ProgramAggregate;
use super::errors::{expression_error, statement_error, SemanticError, SemanticResult};
use super::expressions::ExpressionsVerifier;
use super::insights::Insights;
use super::locals::LocalVariables;
use super::resolvers::NameResolver;

struct StatementsVerifier<'a, 'b, 'c, 'l> {
    func: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
    resolver: &'c NameResolver,
    locals: &'l mut LocalVariables,
}

impl<'a, 'b, 'c, 'l> StatementsVerifier<'a, 'b, 'c, 'l> {
    fn new(
        func: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        resolver: &'c NameResolver,
        locals: &'l mut LocalVariables,
    ) -> Self {
        Self { func, aggregate, resolver, locals }
    }

    fn annotate_type(
        &self,
        t: &ParsedType,
        stmt: &StatementWithPos,
    ) -> SemanticResult<VerifiedType> {
        verify_parsed_type(
            t,
            &self.resolver.get_typenames_resolver(&self.func.defined_at),
        )
        .map_err(SemanticError::add_statement(stmt))
    }

    pub fn generate_block_for_if_branch(
        &mut self,
        statements: &[StatementWithPos],
        insights: &mut Insights,
    ) -> SemanticResult<Vec<VStatement>> {
        let last_existing = self.locals.peek_last_local().cloned();

        let mut res = self.generate_block(statements, insights)?;

        while self.locals.peek_last_local() != last_existing.as_ref() {
            let dropped_var = self.locals.drop_last_local();
            res.push(VStatement::DropLocal { name: dropped_var });
        }

        Ok(res)
    }

    pub fn generate_block(
        &mut self,
        statements: &[StatementWithPos],
        insights: &mut Insights,
    ) -> SemanticResult<Vec<VStatement>> {
        let mut res = vec![];
        
        let emit_stmt = |stmt_res: VStatement| {
            res.push(stmt_res);
        };

        let mut new_insights: Option<Insights> = None;

        for statement in statements {
            if insights.break_or_continue_found && new_insights.is_none() {
                new_insights = Some(insights.clone());
            }

            let stmt_insights = match new_insights {
                Some(ref mut i) => i,
                None => insights,
            };
            self.generate_single(statement, stmt_insights, &emit_stmt)?;
        }

        Ok(res)
    }

    fn check_expr(
        &self,
        expr: &ExprWithPos,
        expected: Option<&VerifiedType>,
        insights: &Insights,
    ) -> SemanticResult<VExprTyped> {
        let expr_verified = ExpressionsVerifier::new(
            self.func,
            self.aggregate,
            self.locals,
            insights,
            self.resolver.get_typenames_resolver(&self.func.defined_at),
            self.resolver.get_functions_resolver(&self.func.defined_at),
        );
        expr_verified.calculate(expr, expected)
    }

    fn generate_if_elif_else(
        &mut self,
        condition: &ExprWithPos,
        if_body_input: &[StatementWithPos],
        elif_bodies_input: &[(ExprWithPos, Vec<StatementWithPos>)],
        else_body_input: &[StatementWithPos],
        insights: &mut Insights,
    ) -> SemanticResult<VStatement> {
        let condition = self.check_expr(condition, Some(&Type::Bool), insights)?;

        let mut insights_of_if_branch = insights.clone();

        let if_body =
            self.generate_block_for_if_branch(if_body_input, &mut insights_of_if_branch)?;

        let else_body = match elif_bodies_input {
            [] => self.generate_block_for_if_branch(else_body_input, insights)?,
            [(first_condition, first_body), other_elifs @ ..] => {
                vec![self.generate_if_elif_else(
                    first_condition,
                    first_body,
                    other_elifs,
                    else_body_input,
                    insights,
                )?]
            }
        };
        insights.merge_with(insights_of_if_branch);

        Ok(VStatement::IfElse { condition, if_body, else_body })
    }

    fn generate_single(
        &mut self,
        statement: &StatementWithPos,
        insights: &mut Insights,
        emit_stmt: &dyn FnMut(VStatement) -> (),
    ) -> SemanticResult<VStatement> {
        let stmt_err = SemanticError::add_statement(statement);

        // TODO: warning probably?
        // if insights.return_found {
        //     return statement_error!(statement, "Not reachable (return already occured)");
        // }

        match &statement.statement {
            Statement::Expr(e) => {
                let expr = self.check_expr(e, None, insights)?;
                emit_stmt(VStatement::Expression(expr));
            },
            Statement::VarDecl(var_type, name) => {
                todo!("Refactor this!");
                let var_type = self.annotate_type(var_type, statement)?;
                self.locals.add_variable(name, &var_type).map_err(stmt_err)?;
                insights.add_uninitialized(name);
            }
            Statement::Assign { left, right } => {
                let mut temp_insights: Insights;

                let left_part_insights: &mut Insights = if let Expr::Identifier(name) = &left.expr {
                    temp_insights = insights.clone();
                    temp_insights.mark_as_initialized(name);
                    &mut temp_insights
                } else if let Expr::OwnFieldAccess { field } = &left.expr {
                    temp_insights = insights.clone();
                    temp_insights.mark_own_field_as_initialized(field);
                    &mut temp_insights
                } else {
                    insights
                };

                let left_calculated = self.check_expr(left, None, left_part_insights)?;
                let right_calculated =
                    self.check_expr(right, Some(&left_calculated.expr_type), insights)?;

                if let Expr::Identifier(name) = &left.expr {
                    // NOW we can finally mark it as initialized, just in case
                    insights.mark_as_initialized(name);
                }

                // TODO: emit error based on left pos
                let (base_object, tuple_indexes) = split_left_part_of_assignment(left_calculated);
                let assign_stmt = match base_object.expr {
                    VExpr::GetVar(name) => {
                        if tuple_indexes.is_empty() {
                            insights.mark_as_initialized(&name);
                        }

                        VStatement::AssignLocal { name, tuple_indexes, value: right_calculated }
                    }
                    VExpr::AccessField { object, field } => {
                        if tuple_indexes.is_empty() {
                            insights.mark_own_field_as_initialized(&field);
                        }

                        VStatement::AssignToField {
                            object: *object,
                            field,
                            tuple_indexes,
                            value: right_calculated,
                        }
                    }
                    VExpr::AccessListItem { list, index } => VStatement::AssignToList {
                        list: *list,
                        index: *index,
                        tuple_indexes,
                        value: right_calculated,
                    },
                    _ => {
                        return statement_error!(
                            statement,
                            "Assigning to temporary value is not allowed!",
                        )
                    }
                };
                emit_stmt(assign_stmt);
            }
            Statement::VarDeclWithAssign(var_type, name, value) => {
                let var_type = self.annotate_type(var_type, statement)?;
                let value = self.check_expr(value, Some(&var_type), insights)?;
                self.locals.add_variable(name, &var_type).map_err(stmt_err)?;

                emit_stmt(VStatement::AssignLocal {
                    name: name.clone(),
                    tuple_indexes: vec![],
                    value,
                });
            }
            Statement::Return(option_e) => {
                if self.func.is_constructor && option_e.is_some() {
                    return statement_error!(statement, "Constructor must return void");
                }
                // TODO: check value of return (wtf does that mean?)
                insights.return_found = true;
                let value = match option_e {
                    Some(e) => self.check_expr(e, Some(&self.func.return_type), insights)?,
                    None => VExprTyped {
                        expr: VExpr::TupleValue(vec![]),
                        expr_type: Type::Tuple(vec![]),
                    }
                };
                
                emit_stmt(VStatement::Return(value));
            }
            Statement::Break if !insights.is_in_loop => {
                return statement_error!(statement, "`break` outside loop")
            }
            Statement::Continue if !insights.is_in_loop => {
                return statement_error!(statement, "`continue` outside loop")
            }
            Statement::Break => {
                insights.break_or_continue_found = true;
                emit_stmt(VStatement::Break);
            }
            Statement::Continue => {
                insights.break_or_continue_found = true;
                emit_stmt(VStatement::Continue);
            }
            Statement::IfElse { condition, if_body, elif_bodies, else_body } => {
                todo!("refactor");
                self.generate_if_elif_else(condition, if_body, elif_bodies, else_body, insights)?;
            }
            Statement::While { condition, body } => {
                let condition = self.check_expr(condition, Some(&Type::Bool), insights)?;

                let mut loop_insights = insights.clone();
                loop_insights.is_in_loop = true;

                let body = self.generate_block(body, &mut loop_insights)?;

                emit_stmt(VStatement::While {condition, body});
            }
            Statement::Foreach { item_name, iterable, body } => {
                let iterable_calculated = self.check_expr(iterable, None, insights)?;
                let iterable_type = iterable_calculated.expr_type.clone();

                let item_type = match &iterable_type {
                    Type::List(i) => i.as_ref().clone(),
                    _ => {
                        return expression_error!(
                            iterable,
                            "List is required in foreach, got {}",
                            iterable_type
                        )
                    }
                };
                // index name is muffled to avoid collisions (@ is used to avoid same user-named variables)
                // TODO: still check that original name does not overlap with anything
                let index_name = format!("{}@index_{}", item_name, statement.pos);
                let iterable_name = format!("{}@iterable_{}", item_name, statement.pos);

                self.locals.add_variable(item_name, &item_type).map_err(&stmt_err)?;
                self.locals.add_variable(&index_name, &Type::Int).map_err(&stmt_err)?;
                self.locals
                    .add_variable(&iterable_name, &iterable_type)
                    .map_err(&stmt_err)?;

                let get_iterable_var = || VExprTyped {
                    expr_type: iterable_type.clone(),
                    expr: VExpr::GetVar(iterable_name.clone()),
                };
                let get_index_var =
                    || VExprTyped { expr_type: Type::Int, expr: VExpr::GetVar(index_name.clone()) };

                let define_item_var =
                    VStatement::DeclareVar { var_type: item_type.clone(), name: item_name.clone() };
                let define_index_var = VStatement::DeclareAndAssignVar {
                    var_type: Type::Int,
                    name: index_name.clone(),
                    value: VExprTyped { expr: VExpr::Int(0), expr_type: VerifiedType::Int },
                };
                let defined_iterator_var = VStatement::DeclareAndAssignVar {
                    var_type: iterable_calculated.expr_type.clone(),
                    name: iterable_name.clone(),
                    value: iterable_calculated,
                };

                // Condition to check if all good
                let condition = VExprTyped {
                    expr_type: Type::Bool,
                    expr: VExpr::ApplyOp {
                        operator: RawOperator::LessInts,
                        operands: vec![
                            get_index_var(),
                            VExprTyped {
                                expr_type: Type::Int,
                                expr: VExpr::CallFunction {
                                    name: SymbolFunc::new_std_method(&iterable_type, "len"),
                                    return_type: Type::Int,
                                    args: vec![get_iterable_var()],
                                },
                            },
                        ],
                    },
                };
                let get_by_index_from_iterable = VExprTyped {
                    expr_type: item_type.clone(),
                    expr: VExpr::AccessListItem {
                        list: Box::new(get_iterable_var()),
                        index: Box::new(get_index_var()),
                    },
                };
                let set_item_statement = VStatement::AssignLocal {
                    name: item_name.clone(),
                    tuple_indexes: vec![],
                    value: get_by_index_from_iterable,
                };

                let increase_index_statement = VStatement::AssignLocal {
                    name: index_name.clone(),
                    tuple_indexes: vec![],
                    value: VExprTyped {
                        expr_type: Type::Int,
                        expr: VExpr::ApplyOp {
                            operator: RawOperator::AddInts,
                            operands: vec![
                                get_index_var(),
                                VExprTyped { expr: VExpr::Int(1), expr_type: VerifiedType::Int },
                            ],
                        },
                    },
                };

                // NOTE: this is the only place which performs the calculations of the body!
                let mut loop_insights = insights.clone();
                loop_insights.is_in_loop = true;
                let mut calculated_body = self.generate_block(body, &mut loop_insights)?;

                calculated_body.insert(0, increase_index_statement);
                calculated_body.insert(0, set_item_statement);

                let mut loop_group = vec![define_item_var, define_index_var, defined_iterator_var];
                loop_group.extend(move_variables_out_of_while(
                    condition,
                    calculated_body,
                    self.locals,
                ));
                loop_group.push(VStatement::DropLocal { name: iterable_name.clone() });
                loop_group.push(VStatement::DropLocal { name: index_name.clone() });
                loop_group.push(VStatement::DropLocal { name: item_name.clone() });

                // Check that they are indeed are dropped
                assert_eq!(self.locals.drop_last_local(), iterable_name);
                assert_eq!(self.locals.drop_last_local(), index_name);
                assert_eq!(self.locals.drop_last_local(), item_name.clone());

                VStatement::LoopGroup(loop_group)
            }

            Statement::SendMessage { .. } => todo!("No SendMessage processing yet!"),
        };
        Ok(verified_statement)
    }
}

pub fn verify_statements(
    og_function: &FunctionDecl,
    func: &RawFunction,
    aggregate: &ProgramAggregate,
    resolver: &NameResolver,
) -> SemanticResult<Vec<VStatement>> {
    let mut locals = LocalVariables::from_function_arguments(&func.args);
    if func.is_constructor {
        // Add this to the insights so that semantic checker assumer that object is already allocated
        // Allocate statement itself is added later on
        locals
            .add_variable("this", &func.return_type)
            .expect("This defined multiple times!");
    }

    let mut gen = StatementsVerifier::new(func, aggregate, resolver, &mut locals);

    let mut insights = Insights::new();
    let mut verified = gen.generate_block(&og_function.statements, &mut insights)?;

    if !insights.return_found {
        // Return statement is a must to perform jump after function end
        // so either add one if return is implicit (constructor and void functions)
        // or raise an error

        if func.is_constructor {
            verified.push(return_statement_for_constructor(func))
        } else if func.return_type == Type::Tuple(vec![]) {
            verified.push(VStatement::Return(VExprTyped {
                expr: VExpr::TupleValue(vec![]),
                expr_type: Type::Tuple(vec![]),
            }));
        } else {
            let error_msg = match &func.method_of {
                Some(t) => format!(
                    "Method `{}` of class `{}` is not guaranteed to return a value",
                    func.short_name, t
                ),
                None => format!(
                    "Function `{}` is not guaranteed to return a value",
                    func.short_name
                ),
            };
            return Err(SemanticError::TopLevelError { pos: og_function.pos, message: error_msg });
        }
    }

    if func.is_constructor {
        let type_fields = &aggregate.types[func.method_of.as_ref().unwrap()].fields;
        for (name, _) in type_fields.iter() {
            if !insights.initialized_own_fields.contains(name) {
                return Err(SemanticError::TopLevelError {
                    pos: og_function.pos,
                    message: format!("Constructor does not initialize field `{}`", name),
                });
            }
        }

        verified.insert(0, allocate_object_for_constructor(func));
    }

    Ok(verified)
}

fn split_left_part_of_assignment(vexpr: VExprTyped) -> (VExprTyped, Vec<usize>) {
    // GetVar and AccessField (and AccessListItem) are considered a base part of the assignment
    // which point to the memory part, which will be updated
    // vec![] part is used as an offset which should be added to the memory address
    if let VExpr::AccessTupleItem { tuple, index } = vexpr.expr {
        let (base, mut indexes) = split_left_part_of_assignment(*tuple);
        indexes.push(index);
        (base, indexes)
    } else {
        (vexpr, vec![])
    }
}

pub fn return_statement_for_constructor(func: &RawFunction) -> VStatement {
    let class_name = func.method_of.as_ref().unwrap();

    VStatement::Return(VExprTyped {
        expr: VExpr::GetVar("this".into()),
        expr_type: Type::Custom(class_name.clone()),
    })
}

fn allocate_object_for_constructor(func: &RawFunction) -> VStatement {
    let class_name = func.method_of.as_ref().unwrap();

    VStatement::DeclareAndAssignVar {
        var_type: Type::Custom(class_name.clone()),
        name: "this".into(),
        value: VExprTyped {
            expr: VExpr::Allocate { typename: class_name.clone() },
            expr_type: Type::Custom(class_name.clone()),
        },
    }
}
