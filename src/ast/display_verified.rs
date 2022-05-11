use std::fmt;

use super::verified::{RawFunction, RawOperator, VExpr, VStatement};

impl fmt::Display for RawFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args_repr: Vec<_> =
            self.args.iter().map(|(name, t)| format!("{} {}", t, name)).collect();
        let locals = if self.locals.is_empty() {
            "".into()
        } else {
            let formatted = self.locals.iter().map(|(s, t)| format!("    {} {};", t, s));
            format!(
                "    // LOCALS START\n{}\n    // LOCALS_END\n\n",
                formatted.collect::<Vec<_>>().join("\n")
            )
        };
        write!(
            f,
            "fn {return} {name} ({args}) {{\n{locals}{body}\n }}",
            return = self.return_type,
            name = self.name,
            args = args_repr.join(", "),
            locals = locals,
            body = self
                .body
                .iter()
                .map(|s| s.display_with_ident(true))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

impl VStatement {
    pub fn display_with_ident(&self, with_ident: bool) -> String {
        let res = match self {
            VStatement::IfElse { condition, if_body, else_body } => {
                format!(
                    "\nif {} {{\n{}\n}} else {{\n{}\n}}\n",
                    condition.expr,
                    if_body
                        .iter()
                        .map(|s| s.display_with_ident(true))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    else_body
                        .iter()
                        .map(|s| s.display_with_ident(true))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            VStatement::While { condition, body } => {
                format!(
                    "\nwhile {} {{\n{}\n}}\n",
                    condition.expr,
                    body.iter()
                        .map(|s| s.display_with_ident(true))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
            VStatement::Break => "break;".into(),
            VStatement::Continue => "continue;".into(),
            VStatement::Return(e) => format!("return {};", e.expr),
            VStatement::AssignLocal { name, tuple_indexes, value } => {
                format!(
                    "{}{} = {};",
                    name,
                    self.show_tuple_indexes(tuple_indexes),
                    value.expr
                )
            }
            VStatement::AssignToField { object, field, tuple_indexes, value } => {
                format!(
                    "{}.{}{} = {};",
                    object.expr,
                    field,
                    self.show_tuple_indexes(tuple_indexes),
                    value.expr
                )
            }
            VStatement::AssignToList { list, index, tuple_indexes, value } => {
                format!(
                    "{}[{}]{} = {};",
                    list.expr,
                    index.expr,
                    self.show_tuple_indexes(tuple_indexes),
                    value.expr
                )
            }
            VStatement::Expression(e) => format!("{};", e.expr),
        };
        if with_ident {
            res.split('\n')
                .map(|s| format!("    {}", s))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            res
        }
    }

    pub fn show_tuple_indexes(&self, tuple_indexes: &[usize]) -> String {
        match tuple_indexes.len() {
            0 => "".into(),
            1 => format!("[{}]", tuple_indexes[0]),
            _ => {
                let indexes_str_vec =
                    tuple_indexes.iter().map(|i| format!("[{}]", i)).collect::<Vec<_>>();
                indexes_str_vec.join("")
            }
        }
    }
}

impl fmt::Display for VExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VExpr::Int(i) => write!(f, "{}", i),
            VExpr::String(i) => write!(f, "\"{}\"", i),
            VExpr::Bool(i) => write!(f, "{}", i),
            VExpr::Float(i) => write!(f, "{}", i),
            VExpr::Dummy(t) => write!(f, "@dummy({})", t),
            // VExpr::MaybeEmpty(i) => write!(f, "Some({})", i.expr),
            VExpr::GetVar(i) => write!(f, "{}", i),
            VExpr::AccessTupleItem { tuple, index } => write!(f, "{}[{}]", tuple.expr, index),
            VExpr::TupleValue(items) => {
                let items_str = items.iter().map(|e| format!("{}", e.expr)).collect::<Vec<_>>();
                write!(f, "({})", items_str.join(", "))
            }
            VExpr::ListValue { items, .. } => {
                let items_str = items.iter().map(|e| format!("{}", e.expr)).collect::<Vec<_>>();
                write!(f, "[{}]", items_str.join(", "))
            }
            VExpr::CompareMaybe { left, right, .. } => {
                write!(f, "@comp_maybe({} = {})", left.expr, right.expr)
            }
            VExpr::ApplyOp { operator, operands } => match operands.len() {
                0 => panic!(" no arguments to operator!"),
                1 => write!(f, "({} {})", operator, operands[0].expr),
                2 => write!(
                    f,
                    "({} {} {})",
                    operands[0].expr, operator, operands[1].expr
                ),
                _ => panic!("3 arguments not supported now!"),
            },
            VExpr::TernaryOp { condition, if_true, if_false } => {
                write!(
                    f,
                    "({} ? {} : {})",
                    condition.expr, if_true.expr, if_false.expr
                )
            }
            VExpr::CallFunction { name, args, .. } => {
                let args_str = args.iter().map(|e| format!("{}", e.expr)).collect::<Vec<_>>();
                write!(f, "{}({})", name, args_str.join(", "))
            }
            VExpr::AccessField { object, field } => {
                write!(f, "{}.{}", object.expr, field)
            }
            VExpr::AccessListItem { list, index } => {
                write!(f, "{}[{}]", list.expr, index.expr)
            }
            VExpr::Allocate { typename } => {
                write!(f, "new {}", typename)
            }
        }
    }
}

impl fmt::Display for RawOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RawOperator::UnaryNegateInt => write!(f, "-"),
            RawOperator::AddInts => write!(f, "+"),
            RawOperator::SubInts => write!(f, "-"),
            RawOperator::MulInts => write!(f, "*"),
            RawOperator::DivInts => write!(f, "/"),
            RawOperator::GreaterInts => write!(f, ">"),
            RawOperator::LessInts => write!(f, "<"),
            RawOperator::EqualInts => write!(f, "=="),
            RawOperator::UnaryNegateFloat => write!(f, "-"),
            RawOperator::AddFloats => write!(f, "+"),
            RawOperator::SubFloats => write!(f, "-"),
            RawOperator::MulFloats => write!(f, "*"),
            RawOperator::DivFloats => write!(f, "/"),
            RawOperator::GreaterFloats => write!(f, ">"),
            RawOperator::LessFloats => write!(f, "<"),
            RawOperator::EqualFloats => write!(f, "=="),
            RawOperator::UnaryNegateBool => write!(f, "!"),
            RawOperator::EqualBools => write!(f, "=="),
            RawOperator::AndBools => write!(f, "&&"),
            RawOperator::OrBools => write!(f, "||"),
            RawOperator::EqualStrings => write!(f, "=="),
            RawOperator::AddStrings => write!(f, "+"),
        }
    }
}
