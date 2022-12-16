use std::rc::Rc;

use crate::runtime::{Class, Param, Runtime, RuntimeError, Value, IR};

fn expected<T>(t: &str) -> Runtime<T> {
    Err(RuntimeError::ExpectedType(t.to_string()))
}

fn at_wrap(length: Value, args: Vec<Value>) -> Runtime<Value> {
    let length = length.as_int();
    match &args[0] {
        Value::Integer(at) => {
            if length == 0 || *at < -length || *at >= length {
                return Err(RuntimeError::Panic("index out of range".to_string()));
            }
            Ok(Value::Integer(at.rem_euclid(length)))
        }
        _ => expected("integer"),
    }
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

    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() == *arg)),
        _ => Ok(Value::Bool(false)),
    });

    class.rc()
}

fn build_string_class() -> Rc<Class> {
    let mut class = Class::new();
    class.add_native("length", vec![], |target, _| {
        Ok(Value::Integer(target.as_string().len() as i64))
    });

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

    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(arg) => Ok(Value::Bool(target.as_string() == *arg)),
        _ => Ok(Value::Bool(false)),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(arg) => Ok(Value::Bool(target.as_string() != *arg)),
        _ => Ok(Value::Bool(true)),
    });

    fn code_at_unchecked(target: Value, mut args: Vec<Value>) -> Runtime<Value> {
        let idx = args.pop().unwrap().as_int();
        let ch = target.as_string().chars().nth(idx as usize).unwrap();
        Ok(Value::Integer(ch as i64))
    }

    class.add_handler(
        "code at:".to_string(),
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::Send("length".to_string(), 0),
            IR::SendNative(at_wrap, 1),
            IR::SelfRef,
            IR::SendNative(code_at_unchecked, 1),
        ],
    );
    class.add_handler(
        "at:".to_string(),
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::Send("length".to_string(), 0),
            IR::SendNative(at_wrap, 1),
            IR::SelfRef,
            IR::SendNative(
                |target, mut args| {
                    let idx = args.pop().unwrap().as_int();
                    let ch = target.as_string().chars().nth(idx as usize).unwrap();
                    Ok(Value::String(Rc::new(ch.to_string())))
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
                |received, args| {
                    if &args[0] == &received {
                        Ok(Value::Unit)
                    } else {
                        Err(RuntimeError::Panic(format!(
                            "expected: {:?} received: {:?}",
                            &args[0], received
                        )))
                    }
                },
                1,
            ),
        ],
    );
    class.add_handler(
        "loop:".to_string(),
        vec![Param::Do],
        vec![IR::Local(0), IR::Send("".to_string(), 0), IR::Loop],
    );
    class.add_handler(
        "string from char code:".to_string(),
        vec![Param::Value],
        vec![IR::SendNative(
            |code, _| match code {
                Value::Integer(int) => Ok(Value::String(Rc::new(
                    char::from_u32(int as u32).unwrap().to_string(),
                ))),
                _ => expected("integer"),
            },
            0,
        )],
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
