use crate::compiler::{CompileError, CompileResult, Compiler};
use crate::frame::{Frame, FrameBuilder};
use crate::ir::IR;
use crate::object_builder::{ObjectBuilder, ParamsBuilder};
use crate::parse_binding::Binding;
use crate::parse_stmt::Stmt;
use crate::parser::ParseError;
use crate::send_builder::{Send, SendBuilder};
use crate::source::Source;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(u64, Source),
    Float(f64, Source),
    String(String, Source),
    Identifier(String, Source),
    Paren(Vec<Stmt>, Source),
    Send(Box<Expr>, Send, Source),
    Object(ObjectBuilder, Source),
    Frame(Frame, Source),
    SelfRef(Source),
}

impl Expr {
    pub fn as_binding(self) -> Result<Binding, ParseError> {
        match self {
            Expr::Identifier(key, source) => Ok(Binding::Identifier(key, source)),
            _ => panic!("invalid set binding"),
        }
    }
    pub fn as_set_in_place(self) -> Result<Stmt, ParseError> {
        match self {
            Expr::Send(target, sender, source) => {
                let binding = target.root_target_binding()?;
                Ok(Stmt::Set(binding, Expr::Send(target, sender, source)))
            }
            _ => panic!("invalid set in place"),
        }
    }
    fn root_target_binding(&self) -> Result<Binding, ParseError> {
        match self {
            Expr::Send(target, _, _) => target.root_target_binding(),
            Expr::Identifier(key, source) => Ok(Binding::Identifier(key.to_string(), *source)),
            _ => panic!("invalid set in place"),
        }
    }

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
            Expr::Paren(body, source) => {
                if body.len() == 0 {
                    return Ok(vec![IR::Constant(Value::Unit)]);
                }
                if body.len() == 1 {
                    if let Stmt::Expr(expr) = &body[0] {
                        return expr.compile(compiler);
                    }
                }
                // (a b) => []{: {} a b}
                let mut do_block = ObjectBuilder::new();
                do_block
                    .add_on(ParamsBuilder::key("".to_string()), body.clone())
                    .unwrap();
                let target = FrameBuilder::new()
                    .build_key("".to_string(), *source)
                    .unwrap();
                let mut send = SendBuilder::new();
                send.add_do("".to_string(), do_block).unwrap();

                send.build(target, *source).unwrap().compile(compiler)
            }
            Expr::Send(target, send, _) => send.compile(compiler, target),
            Expr::Object(builder, _) => builder.compile(compiler, None),
            Expr::Frame(frame, _) => frame.compile(compiler),
            Expr::SelfRef(source) => compiler.get_self(*source),
        }
    }
}
