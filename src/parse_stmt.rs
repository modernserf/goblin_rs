use crate::{
    compiler::{CompileResult, Compiler},
    ir::IR,
    parse_binding::Binding,
    parse_expr::Expr,
};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let(Binding, Expr),
}

impl Stmt {
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Stmt::Expr(expr) => expr.compile(compiler),
            Stmt::Let(binding, expr) => {
                let mut value = expr.compile(compiler)?;
                match binding {
                    Binding::Identifier(name, _) => {
                        let record = compiler.add_let(name.to_string());
                        value.push(IR::Assign(record.index));
                        return Ok(value);
                    }
                }
            }
        }
    }
}
