use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    interpreter::{Eval, Interpreter, RuntimeError},
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

pub fn bool_class(ctx: &mut Interpreter, selector: &str, target: bool, args: &[Value]) -> Eval {
    match selector {
        "assert:" => match &args[0] {
            Value::String(str) => {
                if target {
                    ctx.push(Value::Unit);
                    Eval::Ok
                } else {
                    Eval::Error(RuntimeError::AssertionError(str.to_string()))
                }
            }
            _ => primitive_type_error("string", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

pub fn int_class(ctx: &mut Interpreter, selector: &str, target: i64, args: &[Value]) -> Eval {
    match selector {
        "-" => {
            ctx.push(Value::Integer(-target));
            Eval::Ok
        }
        "+:" => match args[0] {
            Value::Integer(r) => {
                ctx.push(Value::Integer(target + r));
                Eval::Ok
            }
            Value::Float(f) => {
                ctx.push(Value::Float(target as f64 + f));
                Eval::Ok
            }
            _ => primitive_type_error("number", &args[0]),
        },
        "-:" => match args[0] {
            Value::Integer(r) => {
                ctx.push(Value::Integer(target - r));
                Eval::Ok
            }
            Value::Float(f) => {
                ctx.push(Value::Float(target as f64 - f));
                Eval::Ok
            }
            _ => primitive_type_error("number", &args[0]),
        },
        "=:" => match args[0] {
            Value::Integer(r) => {
                ctx.push(Value::Bool(target == r));
                Eval::Ok
            }
            _ => primitive_type_error("integer", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

pub fn float_class(ctx: &mut Interpreter, selector: &str, target: f64, args: &[Value]) -> Eval {
    match selector {
        "-" => {
            ctx.push(Value::Float(-target));
            Eval::Ok
        }
        "+:" => match args[0] {
            Value::Integer(r) => {
                ctx.push(Value::Float(target + r as f64));
                Eval::Ok
            }
            Value::Float(r) => {
                ctx.push(Value::Float(target + r));
                Eval::Ok
            }
            _ => primitive_type_error("number", &args[0]),
        },
        "-:" => match args[0] {
            Value::Integer(r) => {
                ctx.push(Value::Float(target - r as f64));
                Eval::Ok
            }
            Value::Float(r) => {
                ctx.push(Value::Float(target - r));
                Eval::Ok
            }
            _ => primitive_type_error("number", &args[0]),
        },
        "=:" => match args[0] {
            Value::Float(r) => {
                ctx.push(Value::Bool(target == r));
                Eval::Ok
            }
            _ => primitive_type_error("float", &args[0]),
        },
        _ => Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string())),
    }
}

pub fn string_class(
    ctx: &mut Interpreter,
    selector: &str,
    target: &Rc<String>,
    args: &[Value],
) -> Eval {
    match selector {
        "++:" => match &args[0] {
            Value::String(arg) => {
                let concat = format!("{}{}", target, arg);
                ctx.push(Value::String(Rc::new(concat)));
                Eval::Ok
            }
            _ => primitive_type_error("string", &args[0]),
        },
        "=:" => match &args[0] {
            Value::String(r) => {
                ctx.push(Value::Bool(target == r));
                Eval::Ok
            }
            _ => primitive_type_error("integer", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

pub fn cell_class(
    ctx: &mut Interpreter,
    selector: &str,
    target: Rc<RefCell<Value>>,
    mut args: Vec<Value>,
) -> Eval {
    match selector {
        "" => {
            let value = target.deref().borrow().clone();
            ctx.push(value);
            Eval::Ok
        }
        ":" => {
            let arg = std::mem::take(&mut args[0]);
            let mut tgt = target.borrow_mut();
            *tgt = arg;
            ctx.push(Value::Unit);
            Eval::Ok
        }
        _ => does_not_understand(selector),
    }
}

#[allow(unused)]
pub fn cell_module(ctx: &mut Interpreter, selector: &str, mut args: Vec<Value>) -> Eval {
    match selector {
        ":" => {
            let arg = std::mem::take(&mut args[0]);
            let value = Value::Cell(Rc::new(RefCell::new(arg)));
            ctx.push(value);
            Eval::Ok
        }
        _ => does_not_understand(selector),
    }
}
