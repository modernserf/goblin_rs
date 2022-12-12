use crate::binding::Binding;
use crate::compiler::{CompileIR, Compiler};
use crate::frame::{Frame, FrameBuilder};
use crate::ir::IR;
use crate::object::{ObjectBuilder, ParamsBuilder};
use crate::parser::{Parse, ParseError};
use crate::send::{Send, SendBuilder};
use crate::source::Source;
use crate::stmt::Stmt;

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
    If(Box<Expr>, Vec<Stmt>, Vec<Stmt>, Source),
    Try(Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn as_binding(self) -> Parse<Binding> {
        match self {
            Expr::Identifier(key, source) => Ok(Binding::Identifier(key, source)),
            _ => Err(ParseError::InvalidSetBinding),
        }
    }
    pub fn as_set_in_place(self) -> Parse<Stmt> {
        match self {
            Expr::Send(target, sender, source) => {
                let binding = target.root_target_binding()?;
                Ok(Stmt::Set(binding, Expr::Send(target, sender, source)))
            }
            _ => Err(ParseError::InvalidSetInPlace),
        }
    }
    fn root_target_binding(&self) -> Parse<Binding> {
        match self {
            Expr::Send(target, _, _) => target.root_target_binding(),
            Expr::Identifier(key, source) => Ok(Binding::Identifier(key.to_string(), *source)),
            _ => Err(ParseError::InvalidSetInPlace),
        }
    }

    pub fn compile_self_ref(self, compiler: &mut Compiler, binding: &Binding) -> CompileIR {
        match self {
            Expr::Object(builder, _) => builder.compile(compiler, Some(binding)),
            _ => self.compile(compiler),
        }
    }

    pub fn compile(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Expr::Integer(value, _) => Ok(vec![IR::Integer(value as i64)]),
            Expr::Float(value, _) => Ok(vec![IR::Float(value)]),
            Expr::String(value, _) => Ok(vec![IR::String(value)]),
            Expr::Identifier(key, _) => compiler.get(&key),
            Expr::Paren(mut body, source) => {
                if body.len() == 0 {
                    return Ok(vec![IR::Unit]);
                }
                if body.len() == 1 {
                    let stmt = body.pop().unwrap();
                    if let Stmt::Expr(expr) = stmt {
                        return expr.compile(compiler);
                    } else {
                        body.push(stmt)
                    }
                }
                // (a b) => []{: {} a b}
                let mut do_block = ObjectBuilder::new();
                do_block
                    .add_on(ParamsBuilder::key("".to_string()), body)
                    .unwrap();
                let target = FrameBuilder::new()
                    .build_key("".to_string(), source)
                    .unwrap();
                let mut send = SendBuilder::new();
                send.add_do("".to_string(), do_block).unwrap();

                send.build(target, source).unwrap().compile(compiler)
            }
            Expr::If(cond, if_true, if_false, source) => {
                // if x then y else z end -> x{: on {true} y on {false} z}
                let mut do_block = ObjectBuilder::new();
                do_block
                    .add_on(ParamsBuilder::key("true".to_string()), if_true)
                    .unwrap();
                do_block
                    .add_on(ParamsBuilder::key("false".to_string()), if_false)
                    .unwrap();
                let mut send = SendBuilder::new();
                send.add_do("".to_string(), do_block).unwrap();
                send.build(*cond.clone(), source).unwrap().compile(compiler)
            }
            Expr::Send(target, send, _) => send.compile(compiler, *target),
            Expr::Object(builder, _) => builder.compile(compiler, None),
            Expr::Frame(frame, _) => frame.compile(compiler),
            Expr::SelfRef(_) => compiler.get_self(),
            Expr::Try(expr, or_else) => match *expr {
                Expr::Send(target, send, _) => send.compile_try(compiler, *target, *or_else),
                _ => todo!("invalid send"),
            },
        }
    }
}
