use std::ops::Deref;
use std::rc::Rc;

use crate::class::{Body, Class, Param, RcClass};
use crate::ir::IR;
use crate::primitive::Primitive;
use crate::runtime::{Runtime, RuntimeError};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Primitive(Primitive),
    Object(Rc<Object>),
    DoObject {
        class: Rc<Class>,
        parent_offset: usize,
        parent_frame_index: usize,
    },
}

impl Default for Value {
    fn default() -> Self {
        Self::Primitive(Primitive::Unit)
    }
}

impl Value {
    pub fn unit() -> Self {
        Self::Primitive(Primitive::Unit)
    }

    pub fn string(str: &str) -> Self {
        Self::Primitive(Primitive::String(Rc::new(str.to_string())))
    }

    pub fn int(value: i64) -> Self {
        Self::Primitive(Primitive::Integer(value))
    }

    pub fn float(value: f64) -> Self {
        Self::Primitive(Primitive::Float(value))
    }

    pub fn bool(value: bool) -> Self {
        if value {
            Self::Primitive(Primitive::True)
        } else {
            Self::Primitive(Primitive::False)
        }
    }

    pub fn object(class: RcClass, ivars: Vec<Value>) -> Self {
        Self::Object(Object::new(class, ivars).rc())
    }

    fn class(&self) -> RcClass {
        match self {
            Self::Primitive(p) => p.class(),
            Self::Object(obj) => obj.class(),
            Self::DoObject { class, .. } => class.clone(),
        }
    }

    pub fn get_handler(&self, selector: &str, arity: usize) -> Runtime<Handler> {
        let class = self.class();
        if let Some(handler) = class.get(selector) {
            Ok(Handler::new(handler.params(), handler.body()))
        } else if let Some(else_body) = class.get_else() {
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

    pub fn ivar(&self, index: usize) -> Value {
        match self {
            Self::Object(obj) => obj.ivar(index).clone(),
            _ => unreachable!(),
        }
    }

    // TODO: this is only used for constructing frames, can it be eliminated?
    pub fn new_instance(&self, ivars: Vec<Value>) -> Value {
        match self {
            Value::Object(obj) => Value::Object(Object::new(obj.class(), ivars).rc()),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Handler {
    params: Vec<Param>,
    body: Body,
}

impl Handler {
    fn new(params: Vec<Param>, body: Body) -> Self {
        Self { params, body }
    }
    pub fn params(&self) -> Vec<Param> {
        self.params.clone()
    }
    pub fn body(&self) -> Body {
        self.body.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    class: Rc<Class>,
    ivars: Vec<Value>,
}

impl Object {
    fn new(class: Rc<Class>, ivars: Vec<Value>) -> Self {
        Self { class, ivars }
    }

    fn ivar(&self, index: usize) -> Value {
        self.ivars[index].clone()
    }

    fn class(&self) -> Rc<Class> {
        self.class.clone()
    }

    fn rc(self) -> Rc<Object> {
        Rc::new(self)
    }
}
