use std::{cell::RefCell, rc::Rc};

use crate::{
    class::{Object, RcClass},
    interpreter::Eval,
    primitive::{
        bool_class, cell_class, does_not_understand, float_class, int_class, string_class,
    },
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
    Do(RcClass, Rc<Object>, usize),
}

impl Default for Value {
    fn default() -> Self {
        Self::Unit
    }
}

impl Value {
    pub fn string(str: &str) -> Self {
        Self::String(Rc::new(str.to_string()))
    }

    pub fn eval(self) -> Eval {
        Eval::Value(self)
    }

    pub fn send(&self, selector: &str, args: Vec<Value>) -> Eval {
        match self {
            Self::Bool(target) => bool_class(selector, *target, &args),
            Self::Integer(target) => int_class(selector, *target, &args),
            Self::Float(target) => float_class(selector, *target, &args),
            Self::String(target) => string_class(selector, target, &args),
            Self::Cell(target) => cell_class(selector, target.clone(), args),
            Self::Object(obj) => Object::send(obj, selector, args),
            Self::Do(class, parent_object, parent_offset) => {
                Object::send_do_block(class, parent_object, *parent_offset, selector, args)
            }
            _ => does_not_understand(selector),
        }
    }
}
