use std::collections::{HashMap, VecDeque};

use crate::{
    compiler::{CompileResult, Compiler},
    ir::IR,
    object_builder::ObjectBuilder,
    parse_expr::Expr,
    parser::{ParseError, ParseResult},
    source::Source,
};

#[derive(Debug, Clone)]
pub struct Send {
    selector: String,
    target: Box<Expr>,
    args: Vec<SendArg>,
}

impl Send {
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        let mut out = Vec::new();
        // Do args must be processed in two separate phases -- the allocation & the class
        let mut queue = VecDeque::new();
        for arg in self.args.iter() {
            match arg {
                SendArg::Do(builder) => {
                    let (mut allocation, arg) = builder.compile_do(compiler)?;
                    out.append(&mut allocation);
                    queue.push_back(arg);
                }
                _ => {}
            }
        }

        let mut tgt = self.target.compile(compiler)?;
        out.append(&mut tgt);
        let arity = self.args.len();
        for arg in self.args.iter() {
            match arg {
                SendArg::Do(_) => {
                    let mut ir = queue.pop_front().unwrap();
                    out.append(&mut ir);
                }
                SendArg::Value(value) => {
                    let mut result = value.compile(compiler)?;
                    out.append(&mut result);
                }
            }
        }
        out.push(IR::Send(self.selector.to_string(), arity));
        Ok(out)
    }
}

pub struct SendBuilder {
    args: HashMap<String, SendArg>,
}

#[derive(Debug, Clone)]
enum SendArg {
    Value(Expr),
    Do(ObjectBuilder),
}

impl SendBuilder {
    pub fn new() -> Self {
        SendBuilder {
            args: HashMap::new(),
        }
    }
    pub fn unary_op(operator: String, target: Expr, source: Source) -> ParseResult<Expr> {
        Ok(Expr::Send(
            Send {
                selector: operator,
                target: Box::new(target),
                args: vec![],
            },
            source,
        ))
    }

    pub fn binary_op(
        operator: String,
        target: Expr,
        operand: Expr,
        source: Source,
    ) -> ParseResult<Expr> {
        Ok(Expr::Send(
            Send {
                selector: operator,
                target: Box::new(target),
                args: vec![SendArg::Value(operand)],
            },
            source,
        ))
    }

    pub fn build_key(self, key: String, target: Expr, source: Source) -> ParseResult<Expr> {
        if self.args.len() > 0 {
            return Err(ParseError::ExpectedPairGotKey(key));
        }
        Ok(Expr::Send(
            Send {
                selector: key,
                target: Box::new(target),
                args: vec![],
            },
            source,
        ))
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
            args.push(arg);
        }

        Ok(Expr::Send(
            Send {
                selector,
                target: Box::new(target),
                args,
            },
            source,
        ))
    }
}
