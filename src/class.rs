use std::{collections::HashMap, rc::Rc};

use crate::{interpreter::SendEffect, ir::IR, runtime_error::RuntimeError, value::Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    handlers: HashMap<String, Handler>,
    else_handler: Option<Body>,
}

pub type RcClass = Rc<Class>;

impl Class {
    pub fn new() -> Self {
        Class {
            handlers: HashMap::new(),
            else_handler: None,
        }
    }
    // allows overwriting of existing handlers
    pub fn add_handler(&mut self, key: &str, params: Vec<Param>, body: Vec<IR>) {
        self.handlers
            .insert(key.to_string(), Handler(params, Rc::new(body)));
    }
    pub fn add_else(&mut self, body: Vec<IR>) {
        self.else_handler = Some(Rc::new(body));
    }
    fn get(&self, selector: &str) -> Option<&Handler> {
        self.handlers.get(selector)
    }
    pub fn rc(self) -> RcClass {
        Rc::new(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Handler(Vec<Param>, Body);

pub type Body = Rc<Vec<IR>>;

// "real" params here, patterns are expanded in builder
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Value,
    Do,
    Var,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    class: Rc<Class>,
    ivars: Vec<Value>,
}

fn check_args(params: &[Param], args: &[Value]) -> Option<SendEffect> {
    if params.len() != args.len() {
        panic!("param length mismatch")
    }
    for (param, arg) in params.iter().zip(args.iter()) {
        match (param, arg) {
            (Param::Do, Value::Do { .. }) => {}
            (Param::Var, Value::Var(..)) => {}
            (_, Value::Var(..)) => {
                return Some(RuntimeError::invalid_arg("value", arg));
            }
            (_, Value::Do { .. }) => {
                return Some(RuntimeError::invalid_arg("value", arg));
            }
            (_, _) => {}
        }
    }

    None
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
        if let Some(Handler(params, body)) = object.class.get(selector) {
            if let Some(err) = check_args(params, &args) {
                return err;
            }
            SendEffect::Call {
                args,
                selector: selector.to_string(),
                object: object.clone(),
                body: body.clone(),
            }
        } else if let Some(body) = &object.class.else_handler {
            SendEffect::Call {
                args: vec![],
                selector: "else".to_string(),
                object: object.clone(),
                body: body.clone(),
            }
        } else {
            return RuntimeError::does_not_understand(selector);
        }
    }
    pub fn send_do_block(
        class: &RcClass,
        own_offset: usize,
        parent_index: usize,
        selector: &str,
        args: Vec<Value>,
    ) -> SendEffect {
        if let Some(Handler(params, body)) = class.get(selector) {
            if let Some(err) = check_args(params, &args) {
                return err;
            }
            SendEffect::CallDoBlock {
                args,
                own_offset,
                parent_index,
                body: body.clone(),
            }
        } else if let Some(body) = &class.else_handler {
            SendEffect::CallDoBlock {
                args: vec![],
                own_offset,
                parent_index,
                body: body.clone(),
            }
        } else {
            return RuntimeError::does_not_understand(selector);
        }
    }
}
