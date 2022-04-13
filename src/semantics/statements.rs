use crate::ast::*;

use super::aggregate::{ProgramAggregate, RawFunction};
use super::expressions::LightExpressionsGenerator;
use super::light_ast::{LExpr, LExprTyped, LStatement};
use super::resolvers::NameResolver;

pub fn expr_to_lexpr(e: &Expr) -> LExprTyped {
    todo!()
}

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

    pub fn generate(&self, statements: &[Statement]) -> Vec<LStatement> {
        let mut res = vec![];
        self.allocate_object_if_constructor(&mut res);

        for statement in statements {
            res.extend(self.generate_single(statement));
        }
        res
    }

    fn generate_single(&self, statement: &Statement) -> Vec<LStatement> {
        let light_statement = match statement {
            Statement::Expr(e) => LStatement::Expression(expr_to_lexpr(e)),

            Statement::Return(e) => LStatement::Return(expr_to_lexpr(e)),
            Statement::Break => LStatement::Break,
            Statement::Continue => LStatement::Continue,

            Statement::IfElse { condition, ifbody, elsebody } => {
                let condition = expr_to_lexpr(condition);
                let ifbody = self.generate(ifbody);
                let elsebody = self.generate(elsebody);
                LStatement::IfElse { condition, ifbody, elsebody }
            }
            Statement::While { condition, body } => {
                let condition = expr_to_lexpr(condition);
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
    let gen = LightStatementsGenerator::new(scope, aggregate, resolver);
    gen.generate(&og_statements)
}
