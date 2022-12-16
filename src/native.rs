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
    class.add_native("to String", vec![], |target, _| {
        Ok(Value::String(Rc::new(target.as_int().to_string())))
    });

    class.rc()
}

fn build_string_class() -> Rc<Class> {
    let mut class = Class::new();
    class.add_handler(
        "++:".to_string(),
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::Send("to String".to_string(), 0),
            IR::SelfRef,
            IR::SendNative(
                |target, args| match &args[0] {
                    Value::String(str) => Ok(Value::String(Rc::new(format!(
                        "{}{}",
                        target.as_string(),
                        str
                    )))),
                    _ => expected("string"),
                },
                1,
            ),
        ],
    );

    class.add_handler("to String".to_string(), vec![], vec![IR::SelfRef]);

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
  static STRING_CLASS: Rc<Class> = build_string_class();
  static NATIVE_MODULE: Rc<Class> = build_native_module();
}

pub fn int_class() -> Rc<Class> {
    INT_CLASS.with(|c| c.clone())
}

pub fn string_class() -> Rc<Class> {
    STRING_CLASS.with(|c| c.clone())
}

pub fn native_module() -> Value {
    NATIVE_MODULE.with(|c| Value::Object(c.clone(), Rc::new(vec![])))
}
