use std::{collections::HashMap, rc::Rc};

use crate::runtime::{NativeHandlerFn, IR};
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    handlers: HashMap<String, Handler>,
    else_handler: Option<Body>,
}

pub type RcClass = Rc<Class>;
pub type RcObject = Rc<Object>;

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
    pub fn add_constant(&mut self, key: &str, value: Value) {
        self.handlers.insert(
            key.to_string(),
            Handler(vec![], Rc::new(vec![IR::Constant(value)])),
        );
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
    pub fn get(&self, selector: &str) -> Option<&Handler> {
        self.handlers.get(selector)
    }
    pub fn rc(self) -> RcClass {
        Rc::new(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Handler(Vec<Param>, Body);
impl Handler {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    class: Rc<Class>,
    ivars: Vec<Value>,
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

    pub fn rc(self) -> RcObject {
        Rc::new(self)
    }
}
