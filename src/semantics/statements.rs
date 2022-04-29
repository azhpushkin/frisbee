use crate::ast::parsed::*;
use crate::ast::verified::{RawFunction, RawOperator, VExpr, VExprTyped, VStatement};
use crate::symbols::{SymbolFunc, SymbolType};
use crate::types::{verify_parsed_type, ParsedType, Type, VerifiedType};

use super::aggregate::ProgramAggregate;
use super::errors::{expression_error, statement_error, SemanticError, SemanticResult};
use super::expressions::ExpressionsVerifier;
use super::insights::Insights;
use super::resolvers::NameResolver;

struct StatementsVerifier<'a, 'b, 'c> {
    scope: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
    resolver: &'c NameResolver,
}

impl<'a, 'b, 'c> StatementsVerifier<'a, 'b, 'c> {
    fn new(
        scope: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        resolver: &'c NameResolver,
    ) -> Self {
        Self { scope, aggregate, resolver }
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

    pub fn generate_block(
        &mut self,
        statements: &[StatementWithPos],
        insights: &mut Insights,
    ) -> SemanticResult<Vec<VStatement>> {
        let mut res = vec![];

        let last_existing = insights.new_variables.last().cloned();

        for statement in statements {
            res.extend(self.generate_single(statement, insights)?);
        }

        while insights.new_variables.last() != last_existing.as_ref() {
            let dropped_var = insights.drop_last_local();
            res.push(VStatement::DropLocal { name: dropped_var });
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
        let if_body = self.generate_block(if_body_input, insights)?;

        let else_body = match elif_bodies_input {
            [] => self.generate_block(else_body_input, insights)?,
            [(first_condition, first_body), other_elifs @ ..] => {
                let elif_else_body = self.generate_if_elif_else(
                    first_condition,
                    first_body,
                    other_elifs,
                    else_body_input,
                    insights,
                )?;
                vec![elif_else_body]
            }
        };

        Ok(VStatement::IfElse { condition, if_body, else_body })
    }

    fn generate_single(
        &mut self,
        statement: &StatementWithPos,
        insights: &mut Insights,
    ) -> SemanticResult<Vec<VStatement>> {
        let stmt_err = SemanticError::add_statement(statement);
        let verified_statement = match &statement.statement {
            Statement::Expr(e) => VStatement::Expression(self.check_expr(e, None, insights)?),
            Statement::VarDecl(var_type, name) => {
                let var_type = self.annotate_type(var_type, statement)?;

                insights.add_variable(name, &var_type).map_err(stmt_err)?;
                VStatement::DeclareVar { var_type, name: name.clone() }
            }
            Statement::Assign { left, right } => {
                let left_calculated = self.check_expr(left, None, insights)?;
                let right_calculated =
                    self.check_expr(right, Some(&left_calculated.expr_type), insights)?;

                // TODO: emit error based on left pos
                let (base_object, tuple_indexes) = split_left_part_of_assignment(left_calculated);
                match base_object.expr {
                    VExpr::GetVar(name) => {
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

                insights.add_variable(name, &var_type).map_err(stmt_err)?;

                VStatement::DeclareAndAssignVar { var_type, name: name.clone(), value }
            }

            Statement::Return(Some(e)) => {
                // TODO: check value of return
                VStatement::Return(self.check_expr(e, Some(&self.scope.return_type), insights)?)
            }
            Statement::Return(None) => VStatement::Return(VExprTyped {
                expr: VExpr::TupleValue(vec![]),
                expr_type: Type::Tuple(vec![]),
            }),
            Statement::Break => VStatement::Break,
            Statement::Continue => VStatement::Continue,
            Statement::IfElse { condition, if_body, elif_bodies, else_body } => {
                self.generate_if_elif_else(condition, if_body, elif_bodies, else_body, insights)?
            }
            Statement::While { condition, body } => {
                let condition = self.check_expr(condition, Some(&Type::Bool), insights)?;
                let body = self.generate_block(body, insights)?;
                VStatement::While { condition, body }
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

                insights.add_variable(item_name, &item_type).map_err(&stmt_err)?;
                insights.add_variable(&index_name, &Type::Int).map_err(&stmt_err)?;
                insights
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

                let mut calculated_body = self.generate_block(body, insights)?;
                calculated_body.insert(0, increase_index_statement);
                calculated_body.insert(0, set_item_statement);

                // and so on
                return Ok(vec![
                    define_item_var,
                    define_index_var,
                    defined_iterator_var,
                    VStatement::While { condition, body: calculated_body },
                ]);
            }

            Statement::SendMessage { .. } => todo!("No SendMessage processing yet!"),
        };
        Ok(vec![verified_statement])
    }
}

pub fn verify_statements(
    og_statements: &[StatementWithPos],
    scope: &RawFunction,
    aggregate: &ProgramAggregate,
    resolver: &NameResolver,
) -> SemanticResult<Vec<VStatement>> {
    let mut insights = Insights::from_function_arguments(&scope.args);
    let mut gen = StatementsVerifier::new(scope, aggregate, resolver);

    let is_constructor = is_constructor(&scope);

    if is_constructor {
        // Add this to the insights so that semantic checker assumer that object is already allocated
        // Allocate statement itself is added later on
        insights
            .add_variable("this", &scope.return_type)
            .expect("This defined multiple times!");
    }

    let mut verified = gen.generate_block(og_statements, &mut insights)?;

    if is_constructor {
        verified.insert(0, allocate_object_for_constructor(scope));
        verified.push(return_statement_for_constructor(scope))
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

fn is_constructor(scope: &RawFunction) -> bool {
    if scope.method_of.is_some() {
        let first_arg = scope.args.names.get(&0);
        // For each method, first argument is "this", which is implicitly passed
        // So if there is no arguments or first argument is not "this" - then we assume
        // that this method is a constructor

        if first_arg.is_none() || first_arg.unwrap() != "this" {
            return true;
        }
    }
    false
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
