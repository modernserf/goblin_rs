use std::{cell::RefCell, rc::Rc};

use crate::{
    class::{Object, RcClass},
    interpreter::SendEffect,
    primitive::{bool_class, cell_class, float_class, int_class, string_class},
    runtime_error::RuntimeError,
};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(Rc<String>),
    Cell(Rc<RefCell<Value>>),
    Object(Rc<Object>),
    Do {
        class: RcClass,
        own_offset: usize,
        parent_index: usize,
    },
    Var(usize, Box<Value>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Unit
    }
}

impl Value {
    #[allow(unused)]
    pub fn string(str: &str) -> Self {
        Self::String(Rc::new(str.to_string()))
    }

    pub fn eval(self) -> SendEffect {
        SendEffect::Value(self)
    }

    pub fn bool(&self) -> bool {
        match self {
            Value::Bool(val) => *val,
            _ => panic!("expected bool"),
        }
    }
    pub fn integer(&self) -> i64 {
        match self {
            Value::Integer(val) => *val,
            _ => panic!("expected integer"),
        }
    }

    pub fn send(&self, selector: &str, args: Vec<Value>) -> SendEffect {
        match self {
            Self::Bool(_) => Object::send_native(bool_class(), self.clone(), selector, args),
            Self::Integer(_) => Object::send_native(int_class(), self.clone(), selector, args),
            Self::Float(target) => float_class(selector, *target, &args),
            Self::String(target) => string_class(selector, target, &args),
            Self::Cell(target) => cell_class(selector, target.clone(), args),
            Self::Object(obj) => Object::send(obj, selector, args),
            Self::Var(_, parent) => parent.send(selector, args),
            Self::Do {
                class,
                own_offset,
                parent_index,
            } => Object::send_do_block(class, *own_offset, *parent_index, selector, args),
            _ => RuntimeError::does_not_understand(selector),
        }
    }
}
