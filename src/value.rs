use std::ops::Deref;
use std::{cell::RefCell, rc::Rc};

use crate::class::{Body, Class, Object, Param, RcClass};
use crate::primitive::{bool_class, cell_class, float_class, int_class, string_class};
use crate::runtime::{Runtime, RuntimeError, IR};

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
    DoObject(Rc<Object>),
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

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(val) => *val,
            _ => panic!("expected bool"),
        }
    }
    pub fn as_integer(&self) -> i64 {
        match self {
            Value::Integer(val) => *val,
            _ => panic!("expected integer"),
        }
    }
    pub fn as_float(&self) -> f64 {
        match self {
            Value::Float(val) => *val,
            _ => panic!("expected float"),
        }
    }
    pub fn as_string(&self) -> &Rc<String> {
        match self {
            Value::String(str) => str,
            _ => panic!("expected string"),
        }
    }
    pub fn as_cell(&self) -> &Rc<RefCell<Value>> {
        match self {
            Value::Cell(cell) => cell,
            _ => panic!("expected cell"),
        }
    }

    fn class(&self) -> RcClass {
        match self {
            Self::Unit => Class::new().rc(),
            Self::Bool(..) => bool_class(),
            Self::Integer(..) => int_class(),
            Self::Float(..) => float_class(),
            Self::String(..) => string_class(),
            Self::Cell(..) => cell_class(),
            Self::Object(obj) => obj.class(),
            Self::DoObject(obj) => obj.class(),
        }
    }

    pub fn get_handler(&self, selector: &str, arity: usize) -> Runtime<Handler> {
        let class = self.class();
        let is_do_block = match self {
            Self::DoObject(_) => true,
            _ => false,
        };
        if let Some(handler) = class.get(selector) {
            Ok(Handler::new(
                self.clone(),
                handler.params(),
                handler.body(),
                is_do_block,
            ))
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

            Ok(Handler::new(self.clone(), vec![], body, is_do_block))
        } else {
            Err(RuntimeError::DoesNotUnderstand(selector.to_string()))
        }
    }

    pub fn ivar(&self, index: usize) -> Value {
        match self {
            Self::Object(obj) => obj.ivar(index).clone(),
            Self::DoObject(obj) => obj.ivar(index).clone(),
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
    value: Value,
    params: Vec<Param>,
    body: Body,
    is_do_block: bool,
}

impl Handler {
    fn new(value: Value, params: Vec<Param>, body: Body, is_do_block: bool) -> Self {
        Self {
            value,
            params,
            body,
            is_do_block,
        }
    }
    pub fn params(&self) -> Vec<Param> {
        self.params.clone()
    }
    pub fn body(&self) -> Body {
        self.body.clone()
    }
    pub fn is_do_block(&self) -> bool {
        self.is_do_block
    }
    pub fn instance(&self) -> Value {
        self.value.clone()
    }
}
