use std::rc::Rc;

use crate::runtime::{Class, Param, Runtime, RuntimeError, Value, IR};

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
    let mut class = Class::new();
    class.add_handler(
        "expected:received:".to_string(),
        vec![Param::Value, Param::Value],
        vec![
            IR::Local(0),
            IR::Local(1),
            IR::SendNative(
                |_, args| {
                    if &args[0] == &args[1] {
                        Ok(Value::Unit)
                    } else {
                        Err(RuntimeError::Panic(format!(
                            "expected: {:?} received: {:?}",
                            &args[0], &args[1]
                        )))
                    }
                },
                2,
            ),
        ],
    );
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
