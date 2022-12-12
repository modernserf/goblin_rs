use std::rc::Rc;

use crate::class::{Class, RcClass};
use crate::primitive::Primitive;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Primitive(Primitive),
    Object(Rc<Object>),
    DoObject {
        class: RcClass,
        parent_offset: usize,
        parent_frame_index: usize,
    },
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

    pub fn class(&self) -> RcClass {
        match self {
            Self::Primitive(p) => p.class(),
            Self::Object(obj) => obj.class(),
            Self::DoObject { class, .. } => class.clone(),
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
            Value::Object(obj) => Value::object(obj.class(), ivars),
            _ => unreachable!(),
        }
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
