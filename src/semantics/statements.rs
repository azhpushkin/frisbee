use crate::ast::*;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::expressions::LightExpressionsGenerator;
use super::light_ast::{LExpr, LStatement, LExprTyped};
use super::resolvers::NameResolver;

pub fn expr_to_lexpr(e: &Expr) -> LExprTyped {
    todo!()
}

struct LightStatementsGenerator<'a, 'b, 'c> {
    scope: &'a RawFunction,
    aggregate: &'b ProgramAggregate,
    resolver: &'c NameResolver,
    lexpr_generator: LightExpressionsGenerator<'a, 'b, 'c>,
}

impl<'a, 'b, 'c> LightStatementsGenerator<'a, 'b, 'c> {
    fn new(
        scope: &'a RawFunction,
        aggregate: &'b ProgramAggregate,
        resolver: &'c NameResolver,
    ) -> Self {
        todo!("Add variables to scope right away!")
        Self { scope, aggregate, resolver, lexpr_generator: LightExpressionsGenerator::new(scope, aggregate, resolver) }
        
    }

    pub fn generate_light_statements(&self, statements: &[Statement]) -> Vec<LStatement> {
        let mut res = vec![];
        for statement in statements {
            res.extend(self.generate_light_statement(statement));
        }
        res
    }

    fn generate_light_statement(&self, statement: &Statement) -> Vec<LStatement> {
        let light_statement = match statement {
            Statement::Expr(e) => LStatement::Expression(expr_to_lexpr(e)),

            Statement::Return(e) => LStatement::Return(expr_to_lexpr(e)),
            Statement::Break => LStatement::Break,
            Statement::Continue => LStatement::Continue,

            Statement::IfElse { condition, ifbody, elsebody } => {
                let condition = expr_to_lexpr(condition);
                let ifbody = self.generate_light_statements(ifbody);
                let elsebody = self.generate_light_statements(elsebody);
                LStatement::IfElse { condition, ifbody, elsebody }
            }
            Statement::While { condition, body } => {
                let condition = expr_to_lexpr(condition);
                let body = self.generate_light_statements(body);
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

            _ => todo!(),
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
    let x = LightStatementsGenerator::new(scope, aggregate, resolver);
    x.generate_light_statements(og_statements)
}
