use std::cell::RefCell;
use std::rc::Rc;

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

struct StatementsVerifier<'a, 'c> {
    pub func: &'a RawFunction,
    pub aggregate: &'a ProgramAggregate,
    pub resolver: &'c NameResolver,
    pub locals: Rc<RefCell<LocalVariables>>,

    stmt_blocks: Vec<Vec<VStatement>>,
}

impl<'a, 'c> StatementsVerifier<'a, 'c> {
    fn new(
        func: &'a RawFunction,
        aggregate: &'a ProgramAggregate,
        resolver: &'c NameResolver,
        locals: Rc<RefCell<LocalVariables>>,
    ) -> Self {
        Self { func, aggregate, resolver, locals, stmt_blocks: vec![] }
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

    fn emit_stmt(&mut self, stmt: VStatement) {
        self.stmt_blocks.last_mut().unwrap().push(stmt);
    }

    pub fn generate_block(
        &mut self,
        statements: &[StatementWithPos],
        insights: &mut Insights,
    ) -> SemanticResult<Vec<VStatement>> {
        self.stmt_blocks.push(vec![]);

        let mut new_insights: Option<Insights> = None;
        self.locals.borrow_mut().start_new_scope();

        for statement in statements {
            if insights.break_or_continue_found && new_insights.is_none() {
                new_insights = Some(insights.clone());
            }

            let stmt_insights = match new_insights {
                Some(ref mut i) => i,
                None => insights,
            };
            self.generate_single(statement, stmt_insights)?;
        }
        self.locals.borrow_mut().drop_current_scope();

        Ok(self.stmt_blocks.pop().expect("Ordering of blocks pop-push failed"))
    }

    fn check_expr(
        &mut self,
        expr: &ExprWithPos,
        expected: Option<&VerifiedType>,
        insights: &Insights,
    ) -> SemanticResult<VExprTyped> {
        let expr_verified = ExpressionsVerifier::new(
            self.func,
            self.aggregate,
            self.locals.clone(),
            insights,
            self.resolver.get_typenames_resolver(&self.func.defined_at),
            self.resolver.get_functions_resolver(&self.func.defined_at),
        );
        let expr = expr_verified.calculate(expr, expected)?;
        for (temp_name, temp_value) in expr_verified.required_temps.into_inner() {
            self.locals
                .borrow_mut()
                .add_variable_exact(&temp_name, &temp_value.expr_type);
            self.emit_stmt(VStatement::AssignLocal {
                name: temp_name,
                tuple_indexes: vec![],
                value: temp_value,
            });
        }
        Ok(expr)
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

        let if_body = self.generate_block(if_body_input, &mut insights_of_if_branch)?;

        let else_body = match elif_bodies_input {
            [] => self.generate_block(else_body_input, insights)?,
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
    ) -> SemanticResult<()> {
        let stmt_err = SemanticError::add_statement(statement);

        // TODO: warning probably?
        // if insights.return_found {
        //     return statement_error!(statement, "Not reachable (return already occured)");
        // }

        match &statement.statement {
            Statement::Expr(e) => {
                let expr = self.check_expr(e, None, insights)?;
                self.emit_stmt(VStatement::Expression(expr));
            }
            Statement::VarDecl(var_type, name) => {
                let var_type = self.annotate_type(var_type, statement)?;
                self.locals
                    .borrow_mut()
                    .add_variable(name, &var_type)
                    .map_err(stmt_err)?;
                insights.add_uninitialized(name);
            }
            Statement::VarDeclWithAssign(var_type, name, value) => {
                let var_type = self.annotate_type(var_type, statement)?;
                let value = self.check_expr(value, Some(&var_type), insights)?;
                let real_name = self
                    .locals
                    .borrow_mut()
                    .add_variable(name, &var_type)
                    .map_err(stmt_err)?;

                self.emit_stmt(VStatement::AssignLocal {
                    name: real_name,
                    tuple_indexes: vec![],
                    value,
                });
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
                self.emit_stmt(assign_stmt);
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
                    },
                };

                self.emit_stmt(VStatement::Return(value));
            }
            Statement::Break if !insights.is_in_loop => {
                return statement_error!(statement, "`break` outside loop")
            }
            Statement::Continue if !insights.is_in_loop => {
                return statement_error!(statement, "`continue` outside loop")
            }
            Statement::Break => {
                insights.break_or_continue_found = true;
                self.emit_stmt(VStatement::Break);
            }
            Statement::Continue => {
                insights.break_or_continue_found = true;
                self.emit_stmt(VStatement::Continue);
            }
            Statement::IfElse { condition, if_body, elif_bodies, else_body } => {
                let if_else_stmt = self.generate_if_elif_else(
                    condition,
                    if_body,
                    elif_bodies,
                    else_body,
                    insights,
                )?;
                self.emit_stmt(if_else_stmt);
            }
            Statement::While { condition, body } => {
                let condition = self.check_expr(condition, Some(&Type::Bool), insights)?;

                let mut loop_insights = insights.clone();
                loop_insights.is_in_loop = true;

                let body = self.generate_block(body, &mut loop_insights)?;

                self.emit_stmt(VStatement::While { condition, body });
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
                let index_name = format!("{}@_index", item_name);
                let iterable_name = format!("{}@_iterable", item_name);

                self.locals.borrow_mut().start_new_scope();
                let real_item_name = self
                    .locals
                    .borrow_mut()
                    .add_variable(item_name, &item_type)
                    .map_err(&stmt_err)?;
                let real_index_name = self
                    .locals
                    .borrow_mut()
                    .add_variable(&index_name, &Type::Int)
                    .map_err(&stmt_err)?;
                let real_iterable_name = self
                    .locals
                    .borrow_mut()
                    .add_variable(&iterable_name, &iterable_type)
                    .map_err(&stmt_err)?;

                let get_var = |locals: &RefCell<LocalVariables>, name: &str| {
                    let (t, n) = locals.borrow().get_variable(name).unwrap();
                    VExprTyped { expr: VExpr::GetVar(n), expr_type: t.clone() }
                };
                let int_expr = |i: i64| VExprTyped { expr: VExpr::Int(i), expr_type: Type::Int };

                self.emit_stmt(VStatement::AssignLocal {
                    name: real_index_name.clone(),
                    tuple_indexes: vec![],
                    value: int_expr(0),
                });
                self.emit_stmt(VStatement::AssignLocal {
                    name: real_iterable_name.clone(),
                    tuple_indexes: vec![],
                    value: iterable_calculated,
                });

                // Condition to check if all good
                let condition = VExprTyped {
                    expr_type: Type::Bool,
                    expr: VExpr::ApplyOp {
                        operator: RawOperator::LessInts,
                        operands: vec![
                            get_var(&self.locals, &index_name),
                            VExprTyped {
                                expr_type: Type::Int,
                                expr: VExpr::CallFunction {
                                    name: SymbolFunc::new_std_method(&iterable_type, "len"),
                                    return_type: Type::Int,
                                    args: vec![get_var(&self.locals, &iterable_name)],
                                },
                            },
                        ],
                    },
                };
                let get_by_index_from_iterable = VExprTyped {
                    expr_type: item_type.clone(),
                    expr: VExpr::AccessListItem {
                        list: Box::new(get_var(&self.locals, &iterable_name)),
                        index: Box::new(get_var(&self.locals, &index_name)),
                    },
                };
                let set_item_statement = VStatement::AssignLocal {
                    name: real_item_name.clone(),
                    tuple_indexes: vec![],
                    value: get_by_index_from_iterable,
                };

                let increase_index_statement = VStatement::AssignLocal {
                    name: real_index_name.clone(),
                    tuple_indexes: vec![],
                    value: VExprTyped {
                        expr_type: Type::Int,
                        expr: VExpr::ApplyOp {
                            operator: RawOperator::AddInts,
                            operands: vec![get_var(&self.locals, &index_name), int_expr(1)],
                        },
                    },
                };

                // NOTE: this is the only place which performs the calculations of the body!
                let mut loop_insights = insights.clone();
                loop_insights.is_in_loop = true;

                let mut calculated_body = self.generate_block(body, &mut loop_insights)?;

                calculated_body.insert(0, increase_index_statement);
                calculated_body.insert(0, set_item_statement);
                self.emit_stmt(VStatement::While { condition, body: calculated_body });

                self.locals.borrow_mut().drop_current_scope();
            }

            Statement::SendMessage { .. } => todo!("No SendMessage processing yet!"),
        };
        Ok(())
    }
}

pub fn verify_raw_function(
    og_function: &FunctionDecl,
    function_symbol: &SymbolFunc,
    aggregate: &mut ProgramAggregate,
    resolver: &NameResolver,
) -> SemanticResult<()> {
    let func = &aggregate.functions[function_symbol];
    let locals = Rc::new(RefCell::new(LocalVariables::from_function_arguments(
        &func.args,
    )));

    if func.is_constructor {
        // Add this to the insights so that semantic checker assumer that object is already allocated
        // Allocate statement itself is added later on
        locals
            .borrow_mut()
            .add_variable_exact("this", &func.return_type)
            .expect("This defined multiple times!");
    }

    let mut gen = StatementsVerifier::new(func, aggregate, resolver, locals);

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

    let mut all_locals = gen.locals.take().move_all_variables();
    for (arg, _) in func.args.iter() {
        all_locals.remove(arg);
    }

    let raw_func = aggregate.functions.get_mut(function_symbol).unwrap();
    raw_func.body = verified;
    raw_func.locals = all_locals.into_iter().collect();

    Ok(())
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

    VStatement::AssignLocal {
        name: "this".into(),
        tuple_indexes: vec![],
        value: VExprTyped {
            expr: VExpr::Allocate { typename: class_name.clone() },
            expr_type: Type::Custom(class_name.clone()),
        },
    }
}
