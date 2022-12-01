use crate::compiler::{CompileError, CompileResult, Compiler};
use crate::frame::Frame;
use crate::ir::IR;
use crate::object_builder::ObjectBuilder;
use crate::parse_binding::Binding;
use crate::parse_stmt::Stmt;
use crate::send_builder::Send;
use crate::source::Source;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(u64, Source),
    Float(f64, Source),
    String(String, Source),
    Identifier(String, Source),
    Paren(Vec<Stmt>, Source),
    Send(Send, Source),
    Object(ObjectBuilder, Source),
    Frame(Frame, Source),
    SelfRef(Source),
}

impl Expr {
    pub fn compile_self_ref(&self, compiler: &mut Compiler, binding: &Binding) -> CompileResult {
        match self {
            Expr::Object(builder, _) => builder.compile(compiler, Some(binding)),
            _ => self.compile(compiler),
        }
    }

    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Expr::Integer(value, _) => {
                let val = Value::Integer(*value as i64);
                Ok(vec![IR::Constant(val)])
            }
            Expr::Float(value, _) => {
                let val = Value::Float(*value);
                Ok(vec![IR::Constant(val)])
            }
            Expr::String(value, _) => {
                let val = Value::string(value);
                Ok(vec![IR::Constant(val)])
            }
            Expr::Identifier(key, src) => match compiler.get(key) {
                Some(ir) => Ok(vec![ir]),
                None => Err(CompileError::UnknownIdentifier(key.to_string(), *src)),
            },
            Expr::Paren(body, _) => {
                if body.len() == 0 {
                    return Ok(vec![IR::Constant(Value::Unit)]);
                }
                if body.len() == 1 {
                    if let Stmt::Expr(expr) = &body[0] {
                        return expr.compile(compiler);
                    }
                }
                unimplemented!()
            }
            Expr::Send(send, _) => send.compile(compiler),
            Expr::Object(builder, _) => builder.compile(compiler, None),
            Expr::Frame(frame, _) => frame.compile(compiler),
            Expr::SelfRef(source) => compiler.get_self(*source),
        }
    }
}
