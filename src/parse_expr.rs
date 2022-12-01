use std::collections::{HashMap, VecDeque};

use crate::compiler::{CompileError, CompileResult, Compiler};
use crate::frame::Frame;
use crate::ir::IR;
use crate::object_builder::ObjectBuilder;
use crate::parse_binding::Binding;
use crate::parse_stmt::Stmt;
use crate::parser::{ParseError, ParseResult};
use crate::source::Source;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Expr {
    Integer(u64, Source),
    Float(f64, Source),
    String(String, Source),
    Identifier(String, Source),
    Paren(Vec<Stmt>, Source),
    UnaryOp(String, Box<Expr>, Source),
    BinaryOp {
        selector: String,
        target: Box<Expr>,
        operand: Box<Expr>,
        source: Source,
    },
    Send {
        selector: String,
        target: Box<Expr>,
        args: Vec<Expr>,
        source: Source,
    },
    Object(ObjectBuilder, Source),
    Frame(Frame, Source),
    SelfRef(Source),
    DoArg(ObjectBuilder, Source),
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
            Expr::UnaryOp(op, expr, _) => {
                let mut value = expr.compile(compiler)?;
                value.push(IR::Send(op.to_string(), 0));
                Ok(value)
            }
            Expr::BinaryOp {
                selector,
                target,
                operand,
                ..
            } => {
                let mut value = target.compile(compiler)?;
                let mut right = operand.compile(compiler)?;
                value.append(&mut right);
                value.push(IR::Send(selector.to_string(), 1));
                Ok(value)
            }
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
            Expr::Send {
                selector,
                target,
                args,
                ..
            } => {
                let mut out = Vec::new();
                // Do args must be processed in two separate phases -- the allocation & the class
                let mut queue = VecDeque::new();
                for arg in args.iter() {
                    match arg {
                        Expr::DoArg(builder, _) => {
                            let (mut allocation, arg) = builder.compile_do(compiler)?;
                            out.append(&mut allocation);
                            queue.push_back(arg);
                        }
                        _ => {}
                    }
                }

                let mut tgt = target.compile(compiler)?;
                out.append(&mut tgt);
                let arity = args.len();
                for arg in args.iter() {
                    match arg {
                        Expr::DoArg(_, _) => {
                            let mut ir = queue.pop_front().unwrap();
                            out.append(&mut ir);
                        }
                        arg => {
                            let mut result = arg.compile(compiler)?;
                            out.append(&mut result);
                        }
                    }
                }
                out.push(IR::Send(selector.to_string(), arity));
                Ok(out)
            }
            Expr::Object(builder, _) => builder.compile(compiler, None),
            Expr::Frame(frame, _) => frame.compile(compiler),
            Expr::SelfRef(source) => compiler.push_self(*source),
            Expr::DoArg(_, _) => unreachable!(),
        }
    }
}

pub struct SendBuilder {
    args: HashMap<String, SendArg>,
}

#[derive(Debug, Clone)]
pub enum SendArg {
    Value(Expr),
    Do(ObjectBuilder),
}

impl SendBuilder {
    pub fn new() -> Self {
        SendBuilder {
            args: HashMap::new(),
        }
    }
    pub fn build_key(self, key: String, target: Expr, source: Source) -> ParseResult<Expr> {
        if self.args.len() > 0 {
            return Err(ParseError::ExpectedPairGotKey(key));
        }
        Ok(Expr::Send {
            selector: key,
            target: Box::new(target),
            args: Vec::new(),
            source,
        })
    }
    pub fn add_value(&mut self, key: String, value: Expr) -> ParseResult<()> {
        match self.args.insert(key.to_string(), SendArg::Value(value)) {
            Some(_) => Err(ParseError::DuplicateKey(key.to_string())),
            None => Ok(()),
        }
    }
    pub fn add_do(&mut self, key: String, value: ObjectBuilder) -> ParseResult<()> {
        match self.args.insert(key.to_string(), SendArg::Do(value)) {
            Some(_) => Err(ParseError::DuplicateKey(key.to_string())),
            None => Ok(()),
        }
    }
    pub fn build(self, target: Expr, source: Source) -> ParseResult<Expr> {
        let mut selector = String::new();
        let mut args = Vec::new();

        let mut entries = self.args.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (key, arg) in entries {
            selector.push_str(&key);
            selector.push(':');
            match arg {
                SendArg::Value(val) => args.push(val),
                SendArg::Do(val) => args.push(Expr::DoArg(val, source)),
            }
        }

        Ok(Expr::Send {
            selector,
            target: Box::new(target),
            args,
            source,
        })
    }
}
