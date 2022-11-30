use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    interpreter::{Eval, RuntimeError},
    value::Value,
};

pub fn does_not_understand(selector: &str) -> Eval {
    Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string()))
}

fn primitive_type_error(expected: &str, arg: &Value) -> Eval {
    Eval::Error(RuntimeError::PrimitiveTypeError {
        expected: expected.to_string(),
        received: arg.clone(),
    })
}

pub fn bool_class(selector: &str, target: bool, args: &[Value]) -> Eval {
    match selector {
        "assert:" => match &args[0] {
            Value::String(str) => {
                if target {
                    Value::Unit.eval()
                } else {
                    Eval::Error(RuntimeError::AssertionError(str.to_string()))
                }
            }
            _ => primitive_type_error("string", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

pub fn int_class(selector: &str, target: i64, args: &[Value]) -> Eval {
    match selector {
        "-" => Value::Integer(-target).eval(),
        "+:" => match args[0] {
            Value::Integer(r) => Value::Integer(target + r).eval(),
            Value::Float(f) => Value::Float(target as f64 + f).eval(),
            _ => primitive_type_error("number", &args[0]),
        },
        "-:" => match args[0] {
            Value::Integer(r) => Value::Integer(target - r).eval(),
            Value::Float(f) => Value::Float(target as f64 - f).eval(),
            _ => primitive_type_error("number", &args[0]),
        },
        "=:" => match args[0] {
            Value::Integer(r) => Value::Bool(target == r).eval(),
            _ => primitive_type_error("integer", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

pub fn float_class(selector: &str, target: f64, args: &[Value]) -> Eval {
    match selector {
        "-" => Value::Float(-target).eval(),
        "+:" => match args[0] {
            Value::Integer(r) => Value::Float(target + r as f64).eval(),
            Value::Float(r) => Value::Float(target + r).eval(),
            _ => primitive_type_error("number", &args[0]),
        },
        "-:" => match args[0] {
            Value::Integer(r) => Value::Float(target - r as f64).eval(),
            Value::Float(r) => Value::Float(target - r).eval(),
            _ => primitive_type_error("number", &args[0]),
        },
        "=:" => match args[0] {
            Value::Float(r) => Value::Bool(target == r).eval(),
            _ => primitive_type_error("float", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

pub fn string_class(selector: &str, target: &Rc<String>, args: &[Value]) -> Eval {
    match selector {
        "++:" => match &args[0] {
            Value::String(arg) => {
                let concat = format!("{}{}", target, arg);
                Value::String(Rc::new(concat)).eval()
            }
            _ => primitive_type_error("string", &args[0]),
        },
        "=:" => match &args[0] {
            Value::String(r) => Value::Bool(target == r).eval(),
            _ => primitive_type_error("integer", &args[0]),
        },
        "debug" => {
            println!("{}", target);
            Value::Unit.eval()
        }
        _ => does_not_understand(selector),
    }
}

pub fn cell_class(selector: &str, target: Rc<RefCell<Value>>, mut args: Vec<Value>) -> Eval {
    match selector {
        "" => target.deref().borrow().clone().eval(),
        ":" => {
            let arg = std::mem::take(&mut args[0]);
            let mut tgt = target.borrow_mut();
            *tgt = arg;
            Value::Unit.eval()
        }
        _ => does_not_understand(selector),
    }
}

#[allow(unused)]
pub fn cell_module(selector: &str, mut args: Vec<Value>) -> Eval {
    match selector {
        ":" => {
            let arg = std::mem::take(&mut args[0]);
            Value::Cell(Rc::new(RefCell::new(arg))).eval()
        }
        _ => does_not_understand(selector),
    }
}
