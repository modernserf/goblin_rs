use std::rc::Rc;

use crate::runtime::{Class, Param, Runtime, RuntimeError, Value, IR};

fn expected<T>(t: &str) -> Runtime<T> {
    Err(RuntimeError::ExpectedType(t.to_string()))
}

fn build_bool_class() -> Rc<Class> {
    let send_true = {
        let mut class = Class::new();
        class.add_handler(
            ":".to_string(),
            vec![Param::Do],
            vec![IR::Local(0), IR::Send("true".to_string(), 0)],
        );
        IR::Object(class.rc(), 0)
    };
    let send_false = {
        let mut class = Class::new();
        class.add_handler(
            ":".to_string(),
            vec![Param::Do],
            vec![IR::Local(0), IR::Send("false".to_string(), 0)],
        );
        IR::Object(class.rc(), 0)
    };

    let mut class = Class::new();
    class.add_native(
        "false:true:",
        vec![Param::Value, Param::Value],
        |target, mut args| {
            let t = args.pop().unwrap();
            let f = args.pop().unwrap();
            if target.as_bool() {
                Ok(t)
            } else {
                Ok(f)
            }
        },
    );
    class.add_native("!", vec![], |target, _| Ok(Value::Bool(!target.as_bool())));
    class.add_native("&&:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(arg) => Ok(Value::Bool(target.as_bool() && *arg)),
        _ => expected("bool"),
    });
    class.add_native("||:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(arg) => Ok(Value::Bool(target.as_bool() || *arg)),
        _ => expected("bool"),
    });
    class.add_native("^^:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(arg) => Ok(Value::Bool(target.as_bool() ^ *arg)),
        _ => expected("bool"),
    });
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(arg) => Ok(Value::Bool(target.as_bool() == *arg)),
        _ => Ok(Value::Bool(false)),
    });
    class.add_handler(
        "!=:".to_string(),
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::Send("=:".to_string(), 1),
            IR::Send("!".to_string(), 0),
        ],
    );
    class.add_handler(
        ":".to_string(),
        vec![Param::Do],
        vec![
            IR::Local(0),
            send_false,
            send_true,
            IR::SelfRef,
            IR::Send("false:true:".to_string(), 2),
            IR::Send(":".to_string(), 1),
        ],
    );
    class.rc()
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
    class.add_handler(
        "loop:".to_string(),
        vec![Param::Do],
        vec![IR::Local(0), IR::Send("".to_string(), 0), IR::Loop],
    );
    class.rc()
}

thread_local! {
    static BOOL_CLASS: Rc<Class> = build_bool_class();
    static INT_CLASS: Rc<Class> = build_int_class();
    static STRING_CLASS: Rc<Class> = build_string_class();
    static NATIVE_MODULE: Rc<Class> = build_native_module();
}

pub fn bool_class() -> Rc<Class> {
    BOOL_CLASS.with(|c| c.clone())
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
