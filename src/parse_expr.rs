use crate::compiler::{CompileError, CompileResult, Compiler};
use crate::ir::IR;
use crate::source::Source;
use crate::value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Integer(u64, Source),
    Identifier(String, Source),
    UnaryOp(String, Box<Expr>, Source),
    BinaryOp {
        selector: String,
        target: Box<Expr>,
        operand: Box<Expr>,
        source: Source,
    },
}

impl Expr {
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Expr::Integer(value, _) => {
                let val = Value::Integer(*value as i64);
                Ok(vec![IR::Constant(val)])
            }
            Expr::Identifier(key, src) => match compiler.get(key) {
                Some(record) => Ok(vec![IR::Local(record.index)]),
                None => Err(CompileError::UnknownIdentifier(key.to_string(), *src)),
            },
            Expr::UnaryOp(op, expr, _) => {
                let mut value = expr.compile(compiler)?;
                value.push(IR::Send(op.to_string(), 0));
                Ok(value)
            }
            Expr::BinaryOp {
                selector,
                target,
                operand,
                source,
            } => {
                let mut value = target.compile(compiler)?;
                let mut right = operand.compile(compiler)?;
                value.append(&mut right);
                value.push(IR::Send(selector.to_string(), 1));
                Ok(value)
            }
        }
    }
}
