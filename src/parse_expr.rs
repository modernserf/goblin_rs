use crate::compiler::CompileError;
use crate::compiler::CompileResult;
use crate::compiler::Compiler;
use crate::ir::Value;
use crate::ir::IR;
use crate::source::Source;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expr {
    Integer(u64, Source),
}

impl Expr {
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Expr::Integer(value, _) => {
                let val = Value::Integer(*value);
                Ok(vec![IR::Constant(val)])
            }
        }
    }
}
