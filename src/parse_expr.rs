use crate::compiler::{CompileError, CompileResult, Compiler};
use crate::ir::IR;
use crate::source::Source;
use crate::value::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Integer(u64, Source),
    Identifier(String, Source),
}

impl Expr {
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Expr::Integer(value, _) => {
                let val = Value::Integer(*value);
                Ok(vec![IR::Constant(val)])
            }
            Expr::Identifier(key, src) => match compiler.get(key) {
                Some(record) => Ok(vec![IR::Local(record.index)]),
                None => Err(CompileError::UnknownIdentifier(key.to_string(), *src)),
            },
        }
    }
}
