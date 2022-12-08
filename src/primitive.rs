use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    class::{Class, Object, Param, RcClass},
    interpreter::SendEffect,
    ir::IR,
    runtime_error::RuntimeError,
    value::Value,
};

pub fn bool_class(selector: &str, target: bool, args: &[Value]) -> SendEffect {
    match selector {
        "assert:" => match &args[0] {
            Value::String(str) => {
                if target {
                    Value::Unit.eval()
                } else {
                    RuntimeError::assertion_error(str)
                }
            }
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        },
        ":" => {
            let selector = if target { "true" } else { "false" };
            args[0].send(selector, vec![])
        }
        _ => RuntimeError::does_not_understand(selector),
    }
}

fn build_int_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("-", vec![], |it, _| Value::Integer(-it.integer()).eval());
    class.add_native("+:", vec![Param::Value], |target, args| {
        let target = target.integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target + r).eval(),
            Value::Float(f) => Value::Float(target as f64 + f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("-:", vec![Param::Value], |target, args| {
        let target = target.integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target - r).eval(),
            Value::Float(f) => Value::Float(target as f64 - f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Bool(target.integer() == r).eval(),
        _ => RuntimeError::primitive_type_error("integer", &args[0]),
    });
    class.rc()
}

thread_local! {
    static INT_CLASS : RcClass = build_int_class()
}
pub fn int_class() -> RcClass {
    INT_CLASS.with(|c| c.clone())
}

// pub fn int_class(selector: &str, target: i64, args: &[Value]) -> SendEffect {
//     match selector {
//         "-" => Value::Integer(-target).eval(),
//         "+:" => match args[0] {
//             Value::Integer(r) => Value::Integer(target + r).eval(),
//             Value::Float(f) => Value::Float(target as f64 + f).eval(),
//             _ => RuntimeError::primitive_type_error("number", &args[0]),
//         },
//         "-:" => match args[0] {
//             Value::Integer(r) => Value::Integer(target - r).eval(),
//             Value::Float(f) => Value::Float(target as f64 - f).eval(),
//             _ => RuntimeError::primitive_type_error("number", &args[0]),
//         },
//         "=:" => match args[0] {
//             Value::Integer(r) => Value::Bool(target == r).eval(),
//             _ => RuntimeError::primitive_type_error("integer", &args[0]),
//         },
//         _ => RuntimeError::does_not_understand(selector),
//     }
// }

pub fn float_class(selector: &str, target: f64, args: &[Value]) -> SendEffect {
    match selector {
        "-" => Value::Float(-target).eval(),
        "+:" => match args[0] {
            Value::Integer(r) => Value::Float(target + r as f64).eval(),
            Value::Float(r) => Value::Float(target + r).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        },
        "-:" => match args[0] {
            Value::Integer(r) => Value::Float(target - r as f64).eval(),
            Value::Float(r) => Value::Float(target - r).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        },
        "=:" => match args[0] {
            Value::Float(r) => Value::Bool(target == r).eval(),
            _ => RuntimeError::primitive_type_error("float", &args[0]),
        },
        _ => RuntimeError::does_not_understand(selector),
    }
}

pub fn string_class(selector: &str, target: &Rc<String>, args: &[Value]) -> SendEffect {
    match selector {
        "++:" => match &args[0] {
            Value::String(arg) => {
                let concat = format!("{}{}", target, arg);
                Value::String(Rc::new(concat)).eval()
            }
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        },
        "=:" => match &args[0] {
            Value::String(r) => Value::Bool(target == r).eval(),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        },
        "debug" => {
            println!("{}", target);
            Value::Unit.eval()
        }
        _ => RuntimeError::does_not_understand(selector),
    }
}

pub fn cell_class(selector: &str, target: Rc<RefCell<Value>>, mut args: Vec<Value>) -> SendEffect {
    match selector {
        "" => target.deref().borrow().clone().eval(),
        ":" => {
            let arg = std::mem::take(&mut args[0]);
            let mut tgt = target.borrow_mut();
            *tgt = arg;
            Value::Unit.eval()
        }
        _ => RuntimeError::does_not_understand(selector),
    }
}

#[allow(unused)]
pub fn cell_module(selector: &str, mut args: Vec<Value>) -> SendEffect {
    match selector {
        ":" => {
            let arg = std::mem::take(&mut args[0]);
            Value::Cell(Rc::new(RefCell::new(arg))).eval()
        }
        _ => RuntimeError::does_not_understand(selector),
    }
}

fn get_cell_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, mut args| {
        let arg = std::mem::take(&mut args[0]);
        Value::Cell(Rc::new(RefCell::new(arg))).eval()
    });
    let obj = Object::new(class.rc(), vec![Value::Unit]);
    Value::Object(Rc::new(obj))
}

fn get_native_module() -> RcClass {
    let mut class = Class::new();
    class.add_handler("Cell", vec![], vec![IR::Constant(get_cell_module())]);

    class.rc()
}

thread_local! {
    static NATIVE_MODULE : RcClass = get_native_module()
}

pub fn native() -> RcClass {
    NATIVE_MODULE.with(|x| x.clone())
}
