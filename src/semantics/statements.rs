use crate::ast::*;
use crate::types::Type;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::annotations::annotate_type;
use super::expressions::LightExpressionsGenerator;
use super::light_ast::{LExpr, LExprTyped, LStatement};
use super::resolvers::NameResolver;

struct LightStatementsGenerator<'a: 'd, 'b, 'c: 'd, 'd> {
    scope: &'a RawFunction,
    resolver: &'c NameResolver,
    lexpr_generator: LightExpressionsGenerator<'a, 'b, 'd>,
}

impl<'a, 'b, 'c, 'd> LightStatementsGenerator<'a, 'b, 'c, 'd> {
    fn new(
        scope: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        resolver: &'c NameResolver,
    ) -> Self {
        let mut lexpr_generator = LightExpressionsGenerator::new(scope, aggregate, resolver);
        for (name, typename) in scope.args.iter() {
            lexpr_generator.add_variable(name.clone(), typename.clone());
        }

        Self { scope, resolver, lexpr_generator }
    }

    fn annotate_type(&self, t: &Type) -> Type {
        annotate_type(
            t,
            &self.resolver.get_typenames_resolver(&self.scope.defined_at),
        )
    }

    fn is_constructor(&self) -> bool {
        if let Some(class_name) = &self.scope.method_of {
            let first_arg = self.scope.args.names.get(&0);
            // For each method, first argument is "this", which is implicitly passed
            // So if there is no arguments or first argument is not "this" - then we assume
            // that this method is a constructor

            if first_arg.is_none() || first_arg.unwrap() != "this" {
                return true;
            }
        }
        return false;
    }

    fn allocate_object_for_constructor(&self) -> LStatement {
        let class_name = self.scope.method_of.as_ref().unwrap();

        LStatement::DeclareAndAssignVar {
            var_type: class_name.into(),
            name: "this".into(),
            value: LExprTyped {
                expr: LExpr::Allocate { typename: class_name.clone() },
                expr_type: class_name.into(),
            },
        }
    }

    fn return_new_object_for_constructor(&self) -> LStatement {
        let class_name = self.scope.method_of.as_ref().unwrap();

        LStatement::Return(LExprTyped {
            expr: LExpr::GetVar("this".into()),
            expr_type: class_name.into(),
        })
    }

    pub fn generate(&mut self, statements: &[StatementWithPos]) -> Vec<LStatement> {
        let mut res = vec![];

        if self.is_constructor() {
            // Allocate "this" right at the start of the method
            res.push(self.allocate_object_for_constructor());
        }

        for statement in statements {
            res.extend(self.generate_single(&statement.statement));
        }

        if self.is_constructor() {
            // Allocate "this" right at the start of the method
            res.push(self.return_new_object_for_constructor());
        }
        res
    }

    fn check_expr(&self, expr: &ExprWithPos, expected: Option<&Type>) -> LExprTyped {
        self.lexpr_generator.calculate(expr, expected)
    }

    fn generate_if_elif_else(
        &mut self,
        condition: &ExprWithPos,
        if_body_input: &[StatementWithPos],
        elif_bodies_input: &[(ExprWithPos, Vec<StatementWithPos>)],
        else_body_input: &[StatementWithPos],
    ) -> LStatement {
        let condition = self.check_expr(condition, Some(&Type::Bool));
        let if_body = self.generate(if_body_input);
        let else_body;

        if elif_bodies_input.is_empty() {
            else_body = self.generate(else_body_input);
        } else {
            let (first_elif_condition, first_elif_body) = &elif_bodies_input[0];
            let other_elifs = &elif_bodies_input[1..];
            else_body = vec![self.generate_if_elif_else(
                first_elif_condition,
                first_elif_body,
                other_elifs,
                else_body_input,
            )];
        }

        LStatement::IfElse { condition, if_body, else_body }
    }

    fn generate_single(&mut self, statement: &Statement) -> Vec<LStatement> {
        let light_statement = match statement {
            Statement::Expr(e) => LStatement::Expression(self.check_expr(e, None)),
            Statement::VarDecl(var_type, name) => {
                self.lexpr_generator
                    .add_variable(name.clone(), self.annotate_type(var_type));
                LStatement::DeclareVar {
                    var_type: self.annotate_type(var_type),
                    name: name.clone(),
                }
            }
            Statement::Assign { left, right } => {
                let left_calculated = self.check_expr(left, None);
                let right_calculated = self.check_expr(right, Some(&left_calculated.expr_type));
                let is_local = get_local_offset(&left_calculated);
                if let Some((var_name, tuple_indexes)) = is_local {
                    LStatement::AssignLocal {
                        name: var_name,
                        tuple_indexes,
                        value: right_calculated,
                    }
                } else {
                    // We dont need type of left_calculated anymore as it is used only for right typecheck
                    // but expr field is still used, so move it away to new LExpr value
                    match left_calculated.expr {
                        LExpr::AccessField { object, field } => {
                            LStatement::AssignToField { object, field, value: right_calculated }
                        }
                        LExpr::AccessTupleItem { tuple, index } => todo!("Flatten this!"),
                        _ => panic!("Cant assign to {:?}", left_calculated.expr),
                    }
                }
            }
            Statement::VarDeclWithAssign(var_type, name, value) => {
                self.lexpr_generator
                    .add_variable(name.clone(), self.annotate_type(var_type));
                let value = self.check_expr(value, None);
                LStatement::DeclareAndAssignVar {
                    var_type: self.annotate_type(var_type),
                    name: name.clone(),
                    value,
                }
            }

            Statement::Return(Some(e)) => {
                // TODO: check value of return
                LStatement::Return(self.check_expr(e, Some(&self.scope.return_type)))
            }
            Statement::Return(None) => {
                todo!("Return empty tuple here!");
            }
            Statement::Break => LStatement::Break,
            Statement::Continue => LStatement::Continue,
            Statement::IfElse { condition, if_body, elif_bodies, else_body } => {
                self.generate_if_elif_else(condition, if_body, elif_bodies, else_body)
            }
            Statement::While { condition, body } => {
                let condition = self.check_expr(condition, Some(&Type::Bool));
                let body = self.generate(body);
                LStatement::While { condition, body }
            }
            Statement::Foreach { itemname, iterable, body } => {
                // TODO: check that iterable is of type [List]
                let var_type: Type = todo!();
                // TODO: proper random index name generation
                let index_name = format!("{}__{}__index", self.scope.short_name, itemname.clone());

                let add_item_var = LStatement::DeclareVar { var_type, name: itemname.clone() };
                let add_index_var = LStatement::DeclareAndAssignVar {
                    var_type: Type::Int,
                    name: index_name.clone(),
                    value: LExprTyped::int(0),
                };

                // and so on
                return vec![add_item_var, add_index_var];
            }

            f => todo!("not implemented {:?}", f),
        };
        vec![light_statement]
    }
}

pub fn generate_light_statements(
    og_statements: &Vec<StatementWithPos>,
    scope: &RawFunction,
    aggregate: &ProgramAggregate,
    resolver: &NameResolver,
) -> Vec<LStatement> {
    let mut gen = LightStatementsGenerator::new(scope, aggregate, resolver);
    gen.generate(&og_statements)
}

fn get_local_offset(lexpr: &LExprTyped) -> Option<(String, Vec<usize>)> {
    match &lexpr.expr {
        LExpr::GetVar(s) => Some((s.clone(), vec![])),
        LExpr::AccessTupleItem { tuple, index } => {
            let mut res = get_local_offset(tuple.as_ref())?;
            res.1.push(*index);
            Some(res)
        }
        _ => None,
    }
}
