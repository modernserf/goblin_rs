use std::{cell::RefCell, collections::HashMap};

use crate::{
    class::{Class, Handler, Param, RcClass},
    compiler::{CompileResult, Compiler},
    ir::IR,
    parse_expr::Expr,
    parser::{ParseError, ParseResult},
    source::Source,
};

thread_local! {
    static CACHE : RefCell<HashMap<String, RcClass>> = RefCell::new(HashMap::new());
}

fn get_cached_class(selector: &str) -> Option<RcClass> {
    CACHE.with(|cell| {
        let map = cell.borrow();
        map.get(selector).cloned()
    })
}

fn set_cached_class(selector: String, class: RcClass) {
    CACHE.with(|cell| {
        let mut map = cell.borrow_mut();
        map.insert(selector, class);
    });
}

#[derive(Debug, Clone)]
pub enum Frame {
    Key(String),
    Pairs(String, Vec<(String, Expr)>),
}

impl Frame {
    pub fn compile(self, compiler: &mut Compiler) -> CompileResult {
        match self {
            Frame::Key(key) => {
                if let Some(class) = get_cached_class(&key) {
                    return Ok(vec![IR::Object(class, 0)]);
                }

                let mut class = Class::new();
                // matcher
                class.add(
                    ":".to_string(),
                    Handler::on(
                        vec![Param::Do],
                        vec![IR::Local(0), IR::Send(key.to_string(), 0)],
                    ),
                );

                let cls = class.rc();
                set_cached_class(key, cls.clone());
                return Ok(vec![IR::Object(cls, 0)]);
            }
            Frame::Pairs(selector, args) => {
                let arity = args.len();
                let mut out = Vec::new();
                if let Some(class) = get_cached_class(&selector) {
                    // write ivars
                    for (_, val) in args {
                        let mut ivar = val.compile(compiler)?;
                        out.append(&mut ivar);
                    }
                    out.push(IR::Object(class, arity));
                    return Ok(out);
                }

                let mut class = Class::new();

                // matcher
                class.add(
                    ":".to_string(),
                    Handler::on(vec![Param::Do], {
                        let mut body = vec![IR::Local(0)];
                        for i in 0..arity {
                            body.push(IR::IVar(i));
                        }
                        body.push(IR::Send(selector.to_string(), arity));
                        body
                    }),
                );

                for (index, (key, val)) in args.into_iter().enumerate() {
                    // write ivar
                    let mut ivar = val.compile(compiler)?;
                    out.append(&mut ivar);

                    // getter
                    class.add(key.to_string(), Handler::on(vec![], vec![IR::IVar(index)]));

                    // setter
                    class.add(
                        format!("{}:", key),
                        Handler::on(vec![Param::Value], {
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
                            body
                        }),
                    );

                    // updater
                    class.add(
                        format!("-> {}:", key),
                        Handler::on(vec![Param::Do], {
                            let mut body = Vec::new();
                            for i in 0..arity {
                                if i == index {
                                    body.push(IR::Local(0));
                                    body.push(IR::IVar(i));
                                    body.push(IR::Send(":".to_string(), 1));
                                } else {
                                    body.push(IR::IVar(i));
                                }
                            }
                            body.push(IR::SelfObject(arity));
                            body
                        }),
                    );
                }
                let cls = class.rc();
                set_cached_class(selector, cls.clone());
                out.push(IR::Object(cls, arity));
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
