use std::{collections::HashMap, rc::Rc};

use crate::{
    interpreter::{RuntimeError, SendEffect},
    ir::IR,
    primitive::does_not_understand,
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    handlers: HashMap<String, Handler>,
    // else_handler: Option<Handler>,
}

pub type RcClass = Rc<Class>;

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
    pub fn rc(self) -> RcClass {
        Rc::new(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Handler {
    OnHandler(Vec<Param>, Body),
}

impl Handler {
    pub fn on(params: Vec<Param>, body: Vec<IR>) -> Self {
        Handler::OnHandler(params, Rc::new(body))
    }
}

pub type Body = Rc<Vec<IR>>;

// "real" params here, patterns are expanded in builder
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Value,
    Do,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    class: Rc<Class>,
    ivars: Vec<Value>,
}

fn check_args(params: &[Param], args: &[Value]) -> Result<(), RuntimeError> {
    if params.len() != args.len() {
        panic!("param length mismatch")
    }
    for (param, arg) in params.iter().zip(args.iter()) {
        match (param, arg) {
            (Param::Do, Value::Do(..)) => {}
            (_, Value::Do(..)) => {
                return Err(RuntimeError::InvalidArg {
                    expected: "value".to_string(),
                    received: arg.clone(),
                })
            }
            (_, _) => {}
        }
    }

    Ok(())
}

impl Object {
    pub fn new(class: Rc<Class>, ivars: Vec<Value>) -> Self {
        Self { class, ivars }
    }

    pub fn ivar(&self, index: usize) -> Value {
        self.ivars[index].clone()
    }

    pub fn class(&self) -> Rc<Class> {
        self.class.clone()
    }

    pub fn send(object: &Rc<Object>, selector: &str, args: Vec<Value>) -> SendEffect {
        if let Some(handler) = object.class.get(selector) {
            match handler {
                Handler::OnHandler(params, body) => {
                    if let Err(err) = check_args(params, &args) {
                        return SendEffect::Error(err);
                    }
                    SendEffect::Call {
                        args,
                        selector: selector.to_string(),
                        object: object.clone(),
                        body: body.clone(),
                    }
                }
            }
        } else {
            return does_not_understand(selector);
        }
    }
    pub fn send_do_block(
        class: &RcClass,
        parent_object: &Rc<Object>,
        parent_offset: usize,
        selector: &str,
        args: Vec<Value>,
    ) -> SendEffect {
        if let Some(handler) = class.get(selector) {
            match handler {
                Handler::OnHandler(params, body) => {
                    if let Err(err) = check_args(params, &args) {
                        return SendEffect::Error(err);
                    }
                    SendEffect::CallDoBlock {
                        args,
                        parent_object: parent_object.clone(),
                        parent_offset,
                        body: body.clone(),
                    }
                }
            }
        } else {
            return does_not_understand(selector);
        }
    }
}
