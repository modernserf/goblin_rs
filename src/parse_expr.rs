use std::collections::HashMap;
use std::rc::Rc;

use crate::compiler::{CompileError, CompileResult, Compiler};
use crate::ir::IR;
use crate::object_builder::ObjectBuilder;
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
}

impl Expr {
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
                let val = Value::String(Rc::new(value.to_owned()));
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
                let mut value = target.compile(compiler)?;
                let arity = args.len();
                for arg in args.iter() {
                    let mut result = arg.compile(compiler)?;
                    value.append(&mut result);
                }
                value.push(IR::Send(selector.to_string(), arity));
                Ok(value)
            }
            Expr::Object(builder, _) => builder.compile(compiler),
        }
    }
}

pub struct SendBuilder {
    args: HashMap<String, SendArg>,
}

#[derive(Debug, Clone)]
pub enum SendArg {
    Value(Expr),
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
