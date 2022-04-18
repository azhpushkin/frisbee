use crate::ast::*;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::expressions::LightExpressionsGenerator;
use super::light_ast::{LExpr, LExprTyped, LStatement};
use super::resolvers::NameResolver;

struct LightStatementsGenerator<'a: 'd, 'b, 'c: 'd, 'd> {
    scope: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
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

        Self { scope, aggregate, resolver, lexpr_generator }
    }

    fn allocate_object_if_constructor(&self, res: &mut Vec<LStatement>) {
        if let Some(class_name) = &self.scope.method_of {
            let first_arg = self.scope.args.names.get(&0);
            // For each method, first argument is "this", which is implicitly passed
            // So if there is no arguments or first argument is not "this" - then we assume
            // that this method is a constructor

            if first_arg.is_none() || first_arg.unwrap() != "this" {
                // Create "this" right at the start of the method
                res.push(LStatement::DeclareVar {
                    var_type: class_name.into(),
                    name: "this".into(),
                });
                res.push(LStatement::AssignVar {
                    name: "this".into(),
                    value: LExprTyped {
                        expr: LExpr::Allocate { typename: class_name.clone() },
                        expr_type: class_name.into(),
                    },
                });
            }
        }
    }

    pub fn generate(&mut self, statements: &[Statement]) -> Vec<LStatement> {
        let mut res = vec![];
        self.allocate_object_if_constructor(&mut res);

        for statement in statements {
            res.extend(self.generate_single(statement));
        }
        res
    }

    fn check_expr(&self, expr: &ExprWithPos, expected: Option<&Type>) -> LExprTyped {
        self.lexpr_generator.calculate(expr, expected)
    }

    fn generate_if_elif_else(
        &mut self,
        condition: &ExprWithPos,
        if_body_input: &[Statement],
        elif_bodies_input: &[(ExprWithPos, Vec<Statement>)],
        else_body_input: &[Statement],
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
                self.lexpr_generator.add_variable(name.clone(), var_type.clone());
                LStatement::DeclareVar { var_type: var_type.clone(), name: name.clone() }
            }
            Statement::Assign { left, right } => {
                match &left.expr {
                    Expr::Identifier(i) => {
                        // TODO: check assigned value type
                        let value = self.check_expr(right, None);
                        LStatement::AssignVar { name: i.clone(), value }
                    }
                    _ => panic!("Only identifiers now!"),
                }
            }
            Statement::VarDeclWithAssign(var_type, name, value) => {
                self.lexpr_generator.add_variable(name.clone(), var_type.clone());
                let value = self.check_expr(value, None);
                LStatement::DeclareAndAssignVar {
                    var_type: var_type.clone(),
                    name: name.clone(),
                    value,
                }
            }

            Statement::Return(Some(e)) => {
                // TODO: check value of return
                LStatement::Return(self.check_expr(e, self.scope.return_type.as_ref()))
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
                let add_index_var =
                    LStatement::DeclareVar { var_type: Type::Int, name: index_name.clone() };
                let set_index_value =
                    LStatement::AssignVar { name: index_name.clone(), value: LExprTyped::int(0) };

                // and so on
                return vec![add_item_var, add_index_var, set_index_value];
            }

            f => todo!("not implemented {:?}", f),
        };
        vec![light_statement]
    }
}

pub fn generate_light_statements(
    og_statements: &Vec<Statement>,
    scope: &RawFunction,
    aggregate: &ProgramAggregate,
    resolver: &NameResolver,
) -> Vec<LStatement> {
    let mut gen = LightStatementsGenerator::new(scope, aggregate, resolver);
    gen.generate(&og_statements)
}
