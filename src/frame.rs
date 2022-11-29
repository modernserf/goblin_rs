use std::{collections::HashMap, rc::Rc};

use crate::{
    class::{Class, Handler, Param},
    compiler::{CompileResult, Compiler},
    ir::IR,
    parse_expr::Expr,
    parser::{ParseError, ParseResult},
    source::Source,
};

// TODO: use lazy_static for cache

#[derive(Debug, Clone)]
pub enum Frame {
    Key(String),
    Pairs(String, Vec<(String, Expr)>),
}

impl Frame {
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Frame::Key(_) => {
                let class = Class::new();
                // TODO: `:`
                return Ok(vec![IR::Object(Rc::new(class), 0)]);
            }
            Frame::Pairs(_, args) => {
                let mut class = Class::new();
                let mut out = Vec::new();
                let arity = args.len();

                for (index, (key, val)) in args.iter().enumerate() {
                    // write ivar
                    let mut ivar = val.compile(compiler)?;
                    out.append(&mut ivar);

                    // getter
                    class.add(
                        key.to_string(),
                        Handler::OnHandler(vec![], Rc::new(vec![IR::IVar(index)])),
                    );

                    // setter
                    let params = vec![Param::Value];
                    let mut body = Vec::new();
                    // write all ivars to stack, but replace one with the handler arg
                    for i in 0..arity {
                        if i == index {
                            body.push(IR::Local(0));
                        } else {
                            body.push(IR::IVar(i));
                        }
                    }
                    body.push(IR::SelfObject(arity));
                    class.add(
                        format!("{}:", key),
                        Handler::OnHandler(params, Rc::new(body)),
                    )
                }
                out.push(IR::Object(Rc::new(class), arity));
                Ok(out)
            }
        }
    }
}

pub struct FrameBuilder {
    args: HashMap<String, Expr>,
}

impl FrameBuilder {
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
        }
    }
    pub fn build_key(self, key: String, source: Source) -> ParseResult<Expr> {
        if self.args.len() > 0 {
            return Err(ParseError::ExpectedPairGotKey(key));
        }
        return Ok(Expr::Frame(Frame::Key(key), source));
    }
    pub fn add_pair(&mut self, key: String, value: Expr) -> ParseResult<()> {
        if self.args.contains_key(&key) {
            return Err(ParseError::DuplicateKey(key));
        }
        self.args.insert(key, value);
        Ok(())
    }
    pub fn build(self, source: Source) -> ParseResult<Expr> {
        let mut entries = self.args.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        let mut selector = String::new();
        for (key, _) in entries.iter() {
            selector.push_str(&key);
            selector.push(':');
        }
        Ok(Expr::Frame(Frame::Pairs(selector, entries), source))
    }
}
