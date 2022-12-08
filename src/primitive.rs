use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    class::{Class, Object, Param, RcClass},
    interpreter::SendEffect,
    ir::IR,
    runtime_error::RuntimeError,
    value::Value,
};

fn build_bool_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("assert:", vec![Param::Value], |target, args| {
        match &args[0] {
            Value::String(str) => {
                if target.bool() {
                    Value::Unit.eval()
                } else {
                    RuntimeError::assertion_error(str)
                }
            }
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        }
    });
    class.add_native(":", vec![Param::Do], |target, args| {
        let selector = if target.bool() { "true" } else { "false" };
        args[0].send(selector, vec![])
    });
    class.rc()
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

fn build_float_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("=", vec![], |target, args| {
        Value::Float(-target.float()).eval()
    });
    class.add_native("+:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Float(target.float() + r as f64).eval(),
        Value::Float(r) => Value::Float(target.float() + r).eval(),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("-:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Float(target.float() - r as f64).eval(),
        Value::Float(r) => Value::Float(target.float() - r).eval(),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Float(r) => Value::Bool(target.float() == r).eval(),
        _ => RuntimeError::primitive_type_error("float", &args[0]),
    });

    class.rc()
}

pub fn build_string_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("++:", vec![Param::Value], |target, args| {
        // TODO: send `toString` to arg
        match &args[0] {
            Value::String(arg) => {
                let concat = format!("{}{}", target.str(), arg);
                Value::String(Rc::new(concat)).eval()
            }
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        }
    });
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(r) => Value::Bool(target.str() == r).eval(),
        _ => RuntimeError::primitive_type_error("string", &args[0]),
    });
    class.add_native("debug", vec![], |target, _| {
        println!("{}", target.str());
        Value::Unit.eval()
    });
    class.rc()
}

thread_local! {
    static BOOL_CLASS : RcClass = build_bool_class();
    static INT_CLASS : RcClass = build_int_class();
    static FLOAT_CLASS : RcClass = build_float_class();
    static STRING_CLASS : RcClass = build_string_class();
}
pub fn bool_class() -> RcClass {
    BOOL_CLASS.with(|c| c.clone())
}
pub fn int_class() -> RcClass {
    INT_CLASS.with(|c| c.clone())
}
pub fn float_class() -> RcClass {
    FLOAT_CLASS.with(|c| c.clone())
}
pub fn string_class() -> RcClass {
    STRING_CLASS.with(|c| c.clone())
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
