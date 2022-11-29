use std::{collections::HashMap, rc::Rc};

use crate::{
    interpreter::{Eval, Interpreter},
    ir::IR,
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    handlers: HashMap<String, Handler>,
    // else_handler: Option<Handler>,
}

impl Class {
    pub fn new() -> Self {
        Class {
            handlers: HashMap::new(),
            // else_handler: None,
        }
    }
    pub fn add(&mut self, key: String, handler: Handler) {
        self.handlers.insert(key, handler);
    }
    pub fn get(&self, selector: &str) -> Option<&Handler> {
        self.handlers.get(selector)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Handler {
    OnHandler(Vec<Param>, Rc<Vec<IR>>),
}

impl Handler {
    pub fn send(&self, _: &mut Interpreter, args: Vec<Value>, ivars: &[Value]) -> Eval {
        match self {
            Self::OnHandler(params, body) => {
                if params.len() != args.len() {
                    unreachable!("param mismatch")
                }
                Eval::Call {
                    args,
                    ivars: ivars.to_vec(),
                    body: body.clone(),
                }
            }
        }
    }
}

// "real" params here, patterns are expanded in builder
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Value,
}
