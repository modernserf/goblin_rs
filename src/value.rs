use std::{cell::RefCell, rc::Rc};

use crate::{
    class::Object,
    interpreter::{Eval, Interpreter},
    primitive::{does_not_understand, float_class, int_class, string_class},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Integer(i64),
    Float(f64),
    String(Rc<String>),
    Object(Rc<Object>),
    Cell(Rc<RefCell<Value>>),
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

    pub fn send(&self, ctx: &mut Interpreter, selector: &str, args: Vec<Value>) -> Eval {
        match self {
            Self::Integer(target) => int_class(ctx, selector, *target, &args),
            Self::Float(target) => float_class(ctx, selector, *target, &args),
            Self::String(target) => string_class(ctx, selector, target, &args),
            Self::Object(obj) => Object::send(obj, ctx, selector, args),
            _ => does_not_understand(selector),
        }
    }
}
