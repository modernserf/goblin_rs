use crate::{
    ir::{NativeHandlerFn, IR},
    runtime::{Runtime, RuntimeError},
};
use std::{collections::HashMap, ops::Deref, rc::Rc};

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
    pub fn add_native(&mut self, key: &str, params: Vec<Param>, f: NativeHandlerFn) {
        let len = params.len();
        self.handlers.insert(
            key.to_string(),
            Handler(
                params,
                Rc::new(vec![IR::SelfRef, IR::SendPrimitive { f, arity: len }]),
            ),
        );
    }
    pub fn add_else(&mut self, body: Vec<IR>) {
        self.else_handler = Some(Rc::new(body));
    }

    pub fn get_handler(&self, selector: &str, arity: usize) -> Runtime<Handler> {
        if let Some(handler) = self.handlers.get(selector) {
            Ok(handler.clone())
        } else if let Some(else_body) = &self.else_handler {
            let body = {
                // drop args
                let mut out = vec![];
                for _ in 0..arity {
                    out.push(IR::Drop);
                }
                let mut else_body = else_body.deref().clone();
                out.append(&mut else_body);
                Rc::new(out)
            };

            Ok(Handler::new(vec![], body))
        } else {
            Err(RuntimeError::DoesNotUnderstand(selector.to_string()))
        }
    }

    pub fn rc(self) -> RcClass {
        Rc::new(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Handler(Vec<Param>, Body);
impl Handler {
    pub fn new(params: Vec<Param>, body: Body) -> Self {
        Handler(params, body)
    }
    pub fn params(&self) -> Vec<Param> {
        self.0.clone()
    }
    pub fn body(&self) -> Body {
        self.1.clone()
    }
}

pub type Body = Rc<Vec<IR>>;

// "real" params here, patterns are expanded in builder
#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Value,
    Do,
    Var,
}
