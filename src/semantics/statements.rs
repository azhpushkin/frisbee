use crate::ast::parsed::*;
use crate::ast::verified::{RawFunction, RawOperator, VExpr, VExprTyped, VStatement};
use crate::symbols::SymbolFunc;
use crate::types::{verify_parsed_type, ParsedType, Type, VerifiedType};

use super::aggregate::ProgramAggregate;
use super::errors::{expression_error, statement_error, SemanticError, SemanticResult};
use super::expressions::ExpressionsVerifier;
use super::insights::{Insights, LocalVariables};
use super::resolvers::NameResolver;

struct StatementsVerifier<'a, 'b, 'c, 'l> {
    scope: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
    resolver: &'c NameResolver,
    locals: &'l mut LocalVariables,
}

impl<'a, 'b, 'c, 'l> StatementsVerifier<'a, 'b, 'c, 'l> {
    fn new(
        scope: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        resolver: &'c NameResolver,
        locals: &'l mut LocalVariables,
    ) -> Self {
        Self { scope, aggregate, resolver, locals }
    }

    fn annotate_type(
        &self,
        t: &ParsedType,
        stmt: &StatementWithPos,
    ) -> SemanticResult<VerifiedType> {
        verify_parsed_type(
            t,
            &self.resolver.get_typenames_resolver(&self.scope.defined_at),
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

        for statement in statements {
            res.push(self.generate_single(statement, insights)?);
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
            self.scope,
            self.aggregate,
            self.locals,
            insights,
            self.resolver.get_typenames_resolver(&self.scope.defined_at),
            self.resolver.get_functions_resolver(&self.scope.defined_at),
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
    ) -> SemanticResult<VStatement> {
        let stmt_err = SemanticError::add_statement(statement);

        // TODO: warning probably?
        // if insights.return_found {
        //     return statement_error!(statement, "Not reachable (return already occured)");
        // }
        let verified_statement = match &statement.statement {
            Statement::Expr(e) => VStatement::Expression(self.check_expr(e, None, insights)?),
            Statement::VarDecl(var_type, name) => {
                let var_type = self.annotate_type(var_type, statement)?;

                self.locals.add_variable(name, &var_type).map_err(stmt_err)?;
                insights.add_uninitialized(name);

                VStatement::DeclareVar { var_type, name: name.clone() }
            }
            Statement::Assign { left, right } => {
                let left_calculated = if let Expr::Identifier(name) = &left.expr {
                    // If left is just an identifier - temporary mark it as initialized to avoid
                    // warnings about uninitialized variables
                    let was_uninitialized = insights.is_uninitialized(name);
                    insights.mark_as_initialized(name);
                    let calculated = self.check_expr(left, None, insights)?;

                    // Note that we need to unmark it here because it must remain uninitialized for right expr
                    if was_uninitialized {
                        insights.add_uninitialized(name);
                    }

                    calculated
                } else {
                    self.check_expr(left, None, insights)?
                };
                let right_calculated =
                    self.check_expr(right, Some(&left_calculated.expr_type), insights)?;

                if let Expr::Identifier(name) = &left.expr {
                    // NOW we can finally mark it as initialized, just in case
                    insights.mark_as_initialized(name);
                }

                // TODO: emit error based on left pos
                let (base_object, tuple_indexes) = split_left_part_of_assignment(left_calculated);
                match base_object.expr {
                    VExpr::GetVar(name) => {
                        insights.mark_as_initialized(&name);
                        VStatement::AssignLocal { name, tuple_indexes, value: right_calculated }
                    }
                    VExpr::AccessField { object, field } => VStatement::AssignToField {
                        object: *object,
                        field,
                        tuple_indexes,
                        value: right_calculated,
                    },
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
                }
            }
            Statement::VarDeclWithAssign(var_type, name, value) => {
                let var_type = self.annotate_type(var_type, statement)?;
                let value = self.check_expr(value, Some(&var_type), insights)?;

                self.locals.add_variable(name, &var_type).map_err(stmt_err)?;

                VStatement::DeclareAndAssignVar { var_type, name: name.clone(), value }
            }
            Statement::Return(Some(e)) => {
                if self.scope.is_constructor {
                    return statement_error!(statement, "Constructor must return void");
                }
                // TODO: check value of return (wtf does that mean?)
                insights.return_found = true;
                let value = self.check_expr(e, Some(&self.scope.return_type), insights)?;
                VStatement::Return(value)
            }
            Statement::Return(None) => {
                insights.return_found = true;
                VStatement::Return(VExprTyped {
                    expr: VExpr::TupleValue(vec![]),
                    expr_type: Type::Tuple(vec![]),
                })
            }
            Statement::Break if !insights.is_in_loop => {
                return statement_error!(statement, "`break` outside loop")
            }
            Statement::Continue if !insights.is_in_loop => {
                return statement_error!(statement, "`continue` outside loop")
            }
            Statement::Break => VStatement::Break,
            Statement::Continue => VStatement::Continue,
            Statement::IfElse { condition, if_body, elif_bodies, else_body } => {
                self.generate_if_elif_else(condition, if_body, elif_bodies, else_body, insights)?
            }
            Statement::While { condition, body } => {
                let condition = self.check_expr(condition, Some(&Type::Bool), insights)?;

                let mut loop_insights = insights.clone();
                loop_insights.is_in_loop = true;

                let verified_body = self.generate_block(body, &mut loop_insights)?;

                let mut loop_group =
                    move_variables_out_of_while(condition, verified_body, self.locals);
                match &loop_group[..] {
                    [] => unreachable!("at least while loop is always there"),
                    [_] => loop_group.pop().unwrap(),
                    _ => VStatement::LoopGroup(loop_group),
                }
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
    scope: &RawFunction,
    aggregate: &ProgramAggregate,
    resolver: &NameResolver,
) -> SemanticResult<Vec<VStatement>> {
    let mut locals = LocalVariables::from_function_arguments(&scope.args);
    if scope.is_constructor {
        // Add this to the insights so that semantic checker assumer that object is already allocated
        // Allocate statement itself is added later on
        locals
            .add_variable("this", &scope.return_type)
            .expect("This defined multiple times!");
    }

    let mut gen = StatementsVerifier::new(scope, aggregate, resolver, &mut locals);

    let mut insights = Insights::new();
    let mut verified = gen.generate_block(&og_function.statements, &mut insights)?;

    // TODO: return without type should be allowed for constructor
    if !scope.is_constructor && !insights.return_found && scope.return_type != Type::Tuple(vec![]) {
        let error_msg = match &scope.method_of {
            Some(t) => format!(
                "Method `{}` of class `{}` is not guaranteed to return a value",
                scope.short_name, t
            ),
            None => format!(
                "Function `{}` is not guaranteed to return a value",
                scope.short_name
            ),
        };
        return Err(SemanticError::TopLevelError { pos: og_function.pos, message: error_msg });
    }

    if scope.is_constructor {
        verified.insert(0, allocate_object_for_constructor(scope));
        if !insights.return_found {
            // Add return statement, but only if return was not explicitly stated
            // (just to avoid double return as it makes no sense)
            verified.push(return_statement_for_constructor(scope))
        }
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

pub fn return_statement_for_constructor(scope: &RawFunction) -> VStatement {
    let class_name = scope.method_of.as_ref().unwrap();

    VStatement::Return(VExprTyped {
        expr: VExpr::GetVar("this".into()),
        expr_type: Type::Custom(class_name.clone()),
    })
}

fn allocate_object_for_constructor(scope: &RawFunction) -> VStatement {
    let class_name = scope.method_of.as_ref().unwrap();

    VStatement::DeclareAndAssignVar {
        var_type: Type::Custom(class_name.clone()),
        name: "this".into(),
        value: VExprTyped {
            expr: VExpr::Allocate { typename: class_name.clone() },
            expr_type: Type::Custom(class_name.clone()),
        },
    }
}

fn move_variables_out_of_while(
    condition: VExprTyped,
    body: Vec<VStatement>,
    locals: &mut LocalVariables,
) -> Vec<VStatement> {
    let mut declared_variables = vec![];
    let mut new_body = vec![];

    // Iterate over first-level of statements
    for statement in body.into_iter() {
        match statement {
            VStatement::DeclareVar { var_type, name } => {
                declared_variables.push((var_type, name));
            }
            VStatement::DeclareAndAssignVar { var_type, name, value } => {
                let assign_stmt =
                    VStatement::AssignLocal { name: name.clone(), tuple_indexes: vec![], value };
                new_body.push(assign_stmt);
                declared_variables.push((var_type, name));
            }
            s => new_body.push(s),
        };
    }

    let mut loop_group = vec![];

    for (var_type, var_name) in declared_variables.iter().cloned() {
        loop_group.push(VStatement::DeclareVar { var_type, name: var_name });
    }
    loop_group.push(VStatement::While { condition, body: new_body });
    for (_, var_name) in declared_variables.into_iter().rev() {
        assert_eq!(locals.drop_last_local(), var_name, "Locals are FILO");
        loop_group.push(VStatement::DropLocal { name: var_name });
    }

    loop_group
}
