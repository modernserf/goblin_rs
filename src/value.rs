use std::{cell::RefCell, rc::Rc};

use crate::{
    class::Class,
    interpreter::{Eval, Interpreter},
    primitive::{does_not_understand, float_class, int_class, string_class},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Integer(i64),
    Float(f64),
    String(Rc<String>),
    Object(Rc<Class>, IVars),
    Cell(Rc<RefCell<Value>>),
}

impl Default for Value {
    fn default() -> Self {
        Self::Unit
    }
}

pub type IVars = Rc<Vec<Value>>;

impl Value {
    pub fn send(&self, ctx: &mut Interpreter, selector: &str, args: Vec<Value>) -> Eval {
        match self {
            Self::Integer(target) => int_class(ctx, selector, *target, &args),
            Self::Float(target) => float_class(ctx, selector, *target, &args),
            Self::String(target) => string_class(ctx, selector, target, &args),
            Self::Object(cls, ivars) => {
                if let Some(handler) = cls.get(selector) {
                    return handler.send(ctx, args, cls, ivars);
                }
                does_not_understand(selector)
            }
            _ => does_not_understand(selector),
        }
    }
}
