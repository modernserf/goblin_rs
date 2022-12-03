use crate::{
    compiler::{CompileError, CompileIR, Compiler},
    ir::IR,
    parse_binding::Binding,
    parse_expr::Expr,
    value::Value,
};

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Let(Binding, Expr),
    Var(Binding, Expr),
    Set(Binding, Expr),
    Return(Option<Expr>),
}

impl Stmt {
    pub fn compile(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Stmt::Expr(expr) => {
                let mut res = expr.compile(compiler)?;
                res.push(IR::Drop);
                Ok(res)
            }
            Stmt::Let(binding, expr) => {
                let mut value = expr.compile_self_ref(compiler, &binding)?;
                match binding {
                    Binding::Identifier(name, _) => {
                        let record = compiler.add_let(name.to_string());
                        value.push(IR::Assign(record.index));
                        return Ok(value);
                    }
                    Binding::Placeholder(_) => {
                        value.push(IR::Drop);
                        return Ok(value);
                    }
                }
            }
            Stmt::Var(binding, expr) => {
                let mut value = expr.compile(compiler)?;
                match binding {
                    Binding::Identifier(name, _) => {
                        let record = compiler.add_var(name.to_string());
                        value.push(IR::Assign(record.index));
                        return Ok(value);
                    }
                    Binding::Placeholder(source) => Err(CompileError::InvalidVarBinding),
                }
            }
            Stmt::Set(binding, expr) => {
                let mut value = expr.compile(compiler)?;
                match binding {
                    Binding::Identifier(name, src) => {
                        if let Some(index) = compiler.get_var_index(&name) {
                            value.push(IR::Assign(index));
                            return Ok(value);
                        }
                        Err(CompileError::InvalidVarBinding)
                    }
                    Binding::Placeholder(source) => Err(CompileError::InvalidVarBinding),
                }
            }
            Stmt::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let mut ir = expr.compile(compiler)?;
                    ir.push(IR::Return);
                    return Ok(ir);
                } else {
                    let ir = vec![IR::Constant(Value::Unit), IR::Return];
                    return Ok(ir);
                }
            }
        }
    }
}
