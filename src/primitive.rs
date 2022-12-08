use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
    class::{Class, Object, Param, RcClass},
    runtime_error::RuntimeError,
    value::Value,
};

fn build_bool_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("assert:", vec![Param::Value], |target, args| {
        match &args[0] {
            Value::String(str) => {
                if target.as_bool() {
                    Value::Unit.eval()
                } else {
                    RuntimeError::assertion_error(str)
                }
            }
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        }
    });
    class.add_native(":", vec![Param::Do], |target, args| {
        let selector = if target.as_bool() { "true" } else { "false" };
        args[0].send(selector, vec![])
    });
    class.rc()
}

fn build_int_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("-", vec![], |it, _| Value::Integer(-it.as_integer()).eval());
    class.add_native("+:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target + r).eval(),
            Value::Float(f) => Value::Float(target as f64 + f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("-:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target - r).eval(),
            Value::Float(f) => Value::Float(target as f64 - f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Bool(target.as_integer() == r).eval(),
        _ => RuntimeError::primitive_type_error("integer", &args[0]),
    });
    class.rc()
}

fn build_float_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("=", vec![], |target, _| {
        Value::Float(-target.as_float()).eval()
    });
    class.add_native("+:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Float(target.as_float() + r as f64).eval(),
        Value::Float(r) => Value::Float(target.as_float() + r).eval(),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("-:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Float(target.as_float() - r as f64).eval(),
        Value::Float(r) => Value::Float(target.as_float() - r).eval(),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Float(r) => Value::Bool(target.as_float() == r).eval(),
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
                let concat = format!("{}{}", target.as_string(), arg);
                Value::String(Rc::new(concat)).eval()
            }
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        }
    });
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(r) => Value::Bool(target.as_string() == r).eval(),
        _ => RuntimeError::primitive_type_error("string", &args[0]),
    });
    class.add_native("debug", vec![], |target, _| {
        println!("{}", target.as_string());
        Value::Unit.eval()
    });
    class.rc()
}

pub fn build_cell_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("", vec![], |target, _| {
        target.as_cell().deref().borrow().clone().eval()
    });
    class.add_native(":", vec![Param::Value], |target, mut args| {
        let arg = std::mem::take(&mut args[0]);
        let mut tgt = target.as_cell().borrow_mut();
        *tgt = arg;
        Value::Unit.eval()
    });
    class.rc()
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
    class.add_constant("true", Value::Bool(true));
    class.add_constant("false", Value::Bool(false));
    class.add_constant("Cell", get_cell_module());

    class.rc()
}

thread_local! {
    static BOOL_CLASS : RcClass = build_bool_class();
    static INT_CLASS : RcClass = build_int_class();
    static FLOAT_CLASS : RcClass = build_float_class();
    static STRING_CLASS : RcClass = build_string_class();
    static CELL_CLASS : RcClass = build_cell_class();

    static NATIVE_MODULE : RcClass = get_native_module()
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
pub fn cell_class() -> RcClass {
    CELL_CLASS.with(|c| c.clone())
}
pub fn native_module() -> Value {
    NATIVE_MODULE.with(|x| Value::Object(Rc::new(Object::new(x.clone(), vec![]))))
}
