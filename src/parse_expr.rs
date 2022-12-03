use std::rc::Rc;

use crate::compiler::{CompileError, CompileIR, Compiler};
use crate::frame::{Frame, FrameBuilder};
use crate::ir::IR;
use crate::object_builder::{ObjectBuilder, ParamsBuilder};
use crate::parse_binding::Binding;
use crate::parse_error::ParseError;
use crate::parse_stmt::Stmt;
use crate::parser::Parse;
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
    If(Box<Expr>, Vec<Stmt>, Vec<Stmt>, Source),
}

impl Expr {
    pub fn as_binding(self) -> Parse<Binding> {
        match self {
            Expr::Identifier(key, source) => Ok(Binding::Identifier(key, source)),
            _ => ParseError::invalid_set_binding(),
        }
    }
    pub fn as_set_in_place(self) -> Parse<Stmt> {
        match self {
            Expr::Send(target, sender, source) => {
                let binding = target.root_target_binding()?;
                Ok(Stmt::Set(binding, Expr::Send(target, sender, source)))
            }
            _ => ParseError::invalid_set_in_place(),
        }
    }
    fn root_target_binding(&self) -> Parse<Binding> {
        match self {
            Expr::Send(target, _, _) => target.root_target_binding(),
            Expr::Identifier(key, source) => Ok(Binding::Identifier(key.to_string(), *source)),
            _ => ParseError::invalid_set_in_place(),
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
            Expr::Integer(value, _) => {
                let val = Value::Integer(value as i64);
                Ok(vec![IR::Constant(val)])
            }
            Expr::Float(value, _) => {
                let val = Value::Float(value);
                Ok(vec![IR::Constant(val)])
            }
            Expr::String(value, _) => {
                let val = Value::String(Rc::new(value));
                Ok(vec![IR::Constant(val)])
            }
            Expr::Identifier(key, _) => match compiler.get(&key) {
                Some(ir) => Ok(vec![ir]),
                None => Err(CompileError::UnknownIdentifier(key)),
            },
            Expr::Paren(mut body, source) => {
                if body.len() == 0 {
                    return Ok(vec![IR::Constant(Value::Unit)]);
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
        }
    }
}
