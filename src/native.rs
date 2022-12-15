use std::rc::Rc;

use crate::runtime_2::{Class, Param, Runtime, RuntimeError, Value};

fn expected<T>(selector: &str) -> Runtime<T> {
    Err(RuntimeError::ExpectedType(selector.to_string()))
}

fn build_int_class() -> Rc<Class> {
    let mut class = Class::new();

    class.add_native("+:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Integer(target.as_int() + *arg)),
        _ => expected("number"),
    });
    class.add_native("-", vec![], |target, _| {
        Ok(Value::Integer(-target.as_int()))
    });

    class.rc()
}

fn build_native_module() -> Rc<Class> {
    let class = Class::new();
    class.rc()
}

thread_local! {
  static INT_CLASS: Rc<Class> = build_int_class();
  static NATIVE_MODULE: Rc<Class> = build_native_module();
}

pub fn int_class() -> Rc<Class> {
    INT_CLASS.with(|c| c.clone())
}

pub fn native_module() -> Value {
    NATIVE_MODULE.with(|c| Value::Object(c.clone(), Rc::new(vec![])))
}
