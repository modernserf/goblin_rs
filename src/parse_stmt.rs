use crate::{
    compiler::{CompileResult, Compiler},
    parse_expr::Expr,
};

pub enum Stmt {
    Expr(Expr),
}

impl Stmt {
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Stmt::Expr(expr) => expr.compile(compiler),
        }
    }
}
