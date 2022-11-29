use std::{collections::HashMap, rc::Rc};

use crate::{
    interpreter::{Eval, Interpreter},
    ir::IR,
    primitive::does_not_understand,
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
    OnHandler(Vec<Param>, Body),
}

pub type Body = Rc<Vec<IR>>;

// "real" params here, patterns are expanded in builder
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    class: Rc<Class>,
    ivars: Vec<Value>,
}

impl Object {
    pub fn empty() -> Self {
        Self {
            class: Rc::new(Class::new()),
            ivars: Vec::new(),
        }
    }
    pub fn new(class: Rc<Class>, ivars: Vec<Value>) -> Self {
        Self {
            class,
            ivars: ivars,
        }
    }

    pub fn ivar(&self, index: usize) -> Value {
        self.ivars[index].clone()
    }

    pub fn class(&self) -> Rc<Class> {
        self.class.clone()
    }

    pub fn send(
        object: &Rc<Object>,
        _: &mut Interpreter,
        selector: &str,
        args: Vec<Value>,
    ) -> Eval {
        if let Some(handler) = object.class.get(selector) {
            match handler {
                Handler::OnHandler(params, body) => {
                    if params.len() != args.len() {
                        unreachable!("param mismatch")
                    }
                    Eval::Call {
                        args,
                        object: object.clone(),
                        body: body.clone(),
                    }
                }
            }
        } else {
            return does_not_understand(selector);
        }
    }
}
