use std::{cell::RefCell, collections::HashMap};

use crate::{
    class::{Class, Param, RcClass},
    compiler::{CompileIR, Compiler},
    parse_error::ParseError,
    parse_expr::Expr,
    parser::Parse,
    runtime::IR,
    source::Source,
    value::Value,
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
    pub fn compile(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Frame::Key(key) => {
                if let Some(class) = get_cached_class(&key) {
                    return Ok(vec![IR::NewObject { class, arity: 0 }]);
                }

                let mut class = Class::new();
                // matcher
                class.add_handler(":", vec![Param::Do], vec![IR::send(&key, 0)]);
                // fold
                class.add_handler(
                    ":into:",
                    vec![Param::Value, Param::Value],
                    vec![IR::send(&format!("{}:", key), 1)],
                );
                // [x] = other -> other{: on {x} true else false } ? false
                class.add_handler(
                    "=:",
                    vec![Param::Value],
                    vec![
                        // pattern
                        IR::NewObject {
                            class: {
                                let mut class = Class::new();
                                class.add_handler(&key, vec![], vec![IR::Constant(Value::True)]);
                                class.add_else(vec![IR::Constant(Value::False)]);
                                class.rc()
                            },
                            arity: 0,
                        },
                        // or else
                        IR::NewObject {
                            class: {
                                let mut class = Class::new();
                                class.add_handler("", vec![], vec![IR::Constant(Value::False)]);
                                class.rc()
                            },
                            arity: 0,
                        },
                        // target
                        IR::Local { index: 0 },
                        IR::TrySend {
                            selector: ":".to_string(),
                            arity: 1,
                        },
                    ],
                );
                class.add_handler(
                    "!=:",
                    vec![Param::Value],
                    vec![IR::SelfRef, IR::send("=:", 1), IR::send("!", 0)],
                );

                let cls = class.rc();
                set_cached_class(key, cls.clone());
                return Ok(vec![IR::NewObject {
                    class: cls,
                    arity: 0,
                }]);
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
                    out.push(IR::NewObject { class, arity });
                    return Ok(out);
                }

                let mut class = Class::new();

                // matcher
                class.add_handler(":", vec![Param::Do], {
                    let mut body = vec![];
                    for i in 0..arity {
                        body.push(IR::IVar { index: i });
                    }
                    body.push(IR::Local { index: 0 });
                    body.push(IR::send(&selector, arity));
                    body
                });
                // equality
                class.add_handler("=:", vec![Param::Value], {
                    let mut body = vec![];
                    // pattern
                    for i in 0..args.len() {
                        body.push(IR::IVar { index: i });
                    }
                    body.push(IR::NewObject {
                        class: {
                            let mut class = Class::new();
                            class.add_handler(
                                &selector,
                                args.iter().map(|_| Param::Value).collect(),
                                {
                                    let mut body = vec![IR::Constant(Value::True)];
                                    for i in 0..args.len() {
                                        body.push(IR::IVar { index: i });
                                        body.push(IR::Local { index: i });
                                        body.push(IR::send("=:", 1));
                                        body.push(IR::send("&&:", 1));
                                    }
                                    body
                                },
                            );
                            class.add_else(vec![IR::Constant(Value::False)]);
                            class.rc()
                        },
                        arity: args.len(),
                    });
                    //  or else
                    body.push(IR::NewObject {
                        class: {
                            let mut class = Class::new();
                            class.add_handler("", vec![], vec![IR::Constant(Value::False)]);
                            class.rc()
                        },
                        arity: 0,
                    });
                    // target
                    body.push(IR::Local { index: 0 });
                    body.push(IR::TrySend {
                        selector: ":".to_string(),
                        arity: 1,
                    });
                    body
                });
                class.add_handler(
                    "!=:",
                    vec![Param::Value],
                    vec![IR::SelfRef, IR::send("=:", 1), IR::send("!", 0)],
                );

                for (index, (key, val)) in args.into_iter().enumerate() {
                    // write ivar
                    let mut ivar = val.compile(compiler)?;
                    out.append(&mut ivar);

                    // getter
                    class.add_handler(&key, vec![], vec![IR::IVar { index }]);

                    // setter
                    class.add_handler(&format!("{}:", key), vec![Param::Value], {
                        let mut body = Vec::new();
                        // write all ivars to stack, but replace one with the handler arg
                        for i in 0..arity {
                            if i == index {
                                body.push(IR::Local { index: 0 });
                            } else {
                                body.push(IR::IVar { index: i });
                            }
                        }
                        body.push(IR::NewSelf { arity });
                        body
                    });

                    // updater
                    class.add_handler(&format!("-> {}:", key), vec![Param::Do], {
                        let mut body = Vec::new();
                        for i in 0..arity {
                            if i == index {
                                body.push(IR::IVar { index: i });
                                body.push(IR::Local { index: 0 });
                                body.push(IR::send(":", 1));
                            } else {
                                body.push(IR::IVar { index: i });
                            }
                        }
                        body.push(IR::NewSelf { arity });
                        body
                    });
                }
                let cls = class.rc();
                set_cached_class(selector, cls.clone());
                out.push(IR::NewObject { class: cls, arity });
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
    pub fn build_key(self, key: String, source: Source) -> Parse<Expr> {
        if self.args.len() > 0 {
            return ParseError::expected_pair_got_key(&key).map_err(|e| e.with_source(source));
        }
        return Ok(Expr::Frame(Frame::Key(key), source));
    }
    pub fn add_pair(&mut self, key: String, value: Expr) -> Parse<()> {
        if self.args.contains_key(&key) {
            return ParseError::duplicate_key(&key);
        }
        self.args.insert(key, value);
        Ok(())
    }
    pub fn build(self, source: Source) -> Parse<Expr> {
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
