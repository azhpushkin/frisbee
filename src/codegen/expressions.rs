use std::collections::HashMap;

use crate::vm::Op;
use crate::ast::*;
use super::globals::*;


pub struct ExprBytecodeGenerator<'a> {
    globals: &'a Globals,
    locals: &'a HashMap<&'a String, usize>,
    pub bytecode: Vec<u8>
}

impl<'a> ExprBytecodeGenerator<'a> {
    pub fn new(globals: &'a Globals, locals: &'a HashMap<&'a String, usize>) -> Self {
        ExprBytecodeGenerator {
            globals,
            locals,
            bytecode: vec![],
        }
    }
    
    pub fn generate(&mut self, expr: &ExprRaw) {
        match expr {
            ExprRaw::BinOp { left, right, op } => {
                self.generate(left.as_ref());
                self.generate(right.as_ref());
                self.bytecode.push(Op::ADD_INT) // TODO: use types to understand this, based on TypedExpr
            }
            _ => todo!(),
        }
    }

}



#[cfg(test)]
mod tests {
    pub trait ExpressionLike {
        fn get_expr(&self) -> &ExpressionRaw;
        fn get_type(&self) -> &i32;
    }

    enum ExpressionRaw {
        Foo1,
        Foo2(Box<dyn ExpressionLike>),
    }

    struct TypedExpression {
        t: i32,
        raw: Box<ExpressionRaw>,
    }

    enum St<T> where T: ExpressionLike {
        St1,
        St2(Box<T>)
    }

    impl ExpressionLike for TypedExpression {
        fn get_expr(&self) -> &ExpressionRaw {
            self.raw.as_ref()
        }
        fn get_type(&self) -> &i32 {
            &self.t
        }
    }

    impl ExpressionLike for ExpressionRaw {
        fn get_expr(&self) -> &ExpressionRaw {
            &self
        }
        fn get_type(&self) -> &i32 {
            &42
        }
    }

    fn calculate(foo: &ExpressionRaw) -> TypedExpression {
        match foo.as_ref() {
            Foo::Foo1 => Box::new(TypedFoo(foo, 1)),
            Foo::Foo2(inner_foo) => {
                let inner_foo = convert(inner_foo);
                Box::new(TypedFoo(inner_foo, 2))
            },
            Foo::TypedFoo(..) => unreachable!()
        }
    }
    
    fn test() {
        let stmt: St<Expression = St::St2(Box::new(ExpressionRaw::Foo1));

        let vec: Vec<St> = vec![];
    }
}