use std::{cell::RefCell, rc::Rc};

use crate::runtime::{Class, MoreFn, Param, Runtime, RuntimeError, Value, IR};

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
    class.add(
        "!=:",
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::send("=:", 1),
            IR::send("!", 0),
        ],
    );
    class.add(
        ":",
        vec![Param::Do],
        vec![IR::Local(0), IR::SelfRef, IR::SendBool],
    );
    class.rc()
}

fn build_int_class() -> Rc<Class> {
    let mut class = Class::new();

    class.add_native("+:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Integer(target.as_int() + *arg)),
        _ => expected("number"),
    });
    class.add_native("-:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Integer(target.as_int() - *arg)),
        _ => expected("number"),
    });
    class.add_native("%:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Integer(target.as_int().rem_euclid(*arg))),
        _ => expected("number"),
    });
    class.add_native("-", vec![], |target, _| {
        Ok(Value::Integer(-target.as_int()))
    });
    class.add_native("to String", vec![], |target, _| {
        Ok(Value::String(Rc::new(target.as_int().to_string())))
    });
    // value comparison
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() == *arg)),
        _ => Ok(Value::Bool(false)),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() != *arg)),
        _ => Ok(Value::Bool(true)),
    });
    // numeric comparison
    class.add_native("<:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() < *arg)),
        _ => expected("number"),
    });
    class.add_native("<=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() <= *arg)),
        _ => expected("number"),
    });
    class.add_native("==:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() == *arg)),
        _ => expected("number"),
    });
    class.add_native("<>:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() != *arg)),
        _ => expected("number"),
    });
    class.add_native(">=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() >= *arg)),
        _ => expected("number"),
    });
    class.add_native(">:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(arg) => Ok(Value::Bool(target.as_int() > *arg)),
        _ => expected("number"),
    });

    class.rc()
}

fn build_string_class() -> Rc<Class> {
    let mut class = Class::new();
    class.add_native("length", vec![], |target, _| {
        Ok(Value::Integer(target.as_string().len() as i64))
    });

    class.add(
        "++:",
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::send("to String", 0),
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

    class.add(
        "code at:",
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::send("length", 0),
            IR::SendNative(at_wrap, 1),
            IR::SelfRef,
            IR::SendNative(code_at_unchecked, 1),
        ],
    );
    class.add(
        "at:",
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::send("length", 0),
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

    class.add("to String", vec![], vec![IR::SelfRef]);

    class.rc()
}

fn build_array_class() -> Rc<Class> {
    let mut class = Class::new();
    class.add_native("length", vec![], |target, _| {
        Ok(Value::Integer(target.as_array().borrow().len() as i64))
    });
    class.add_native("push:", vec![Param::Value], |target, mut args| {
        target.as_array().borrow_mut().push(args.pop().unwrap());
        Ok(Value::Unit)
    });
    class.add(
        "at:",
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::send("length", 0),
            IR::SendNative(at_wrap, 1),
            IR::SelfRef,
            IR::SendNative(
                |target, mut args| {
                    let idx = args.pop().unwrap().as_int();
                    let value = target.as_array().borrow()[idx as usize].clone();
                    Ok(value)
                },
                1,
            ),
        ],
    );
    class.add(
        "at:value:",
        vec![Param::Value, Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::send("length", 0),
            IR::SendNative(at_wrap, 1),
            IR::Local(1),
            IR::SelfRef,
            IR::SendNative(
                |target, mut args| {
                    let value = args.pop().unwrap();
                    let idx = args.pop().unwrap().as_int();
                    target.as_array().borrow_mut()[idx as usize] = value;
                    Ok(Value::Unit)
                },
                2,
            ),
        ],
    );
    class.add(
        "from:to:",
        vec![Param::Value, Param::Value],
        vec![
            // check from arg
            IR::Local(0),
            IR::SelfRef,
            IR::send("length", 0),
            IR::SendNative(at_wrap, 1),
            // TODO: check to arg
            IR::Local(1),
            IR::SelfRef,
            IR::SendNative(
                |target, mut args| {
                    let to = args.pop().unwrap().as_int() as usize;
                    let from = args.pop().unwrap().as_int() as usize;
                    let slice = target.as_array().borrow()[from..to].to_vec();
                    Ok(Value::MutArray(Rc::new(RefCell::new(slice))))
                },
                2,
            ),
        ],
    );
    class.add(
        "from:",
        vec![Param::Value],
        vec![
            IR::Local(0),
            IR::SelfRef,
            IR::send("length", 0),
            IR::SelfRef,
            IR::send("from:to:", 2),
        ],
    );
    // todo
    class.rc()
}

fn build_native_module() -> Rc<Class> {
    let mut class = Class::new();
    class.add(
        "expected:received:",
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
    class.add(
        "loop:",
        vec![Param::Do],
        vec![IR::Local(0), IR::send("", 0), IR::Loop],
    );
    class.add(
        "string from char code:",
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
    class.add("new Array", vec![], vec![IR::MutArray]);
    class.add_native("debug:", vec![Param::Value], |_, args| {
        println!("{:?}", args[0]);
        Ok(Value::Unit)
    });
    class.rc()
}

thread_local! {
    static BOOL_CLASS: Rc<Class> = build_bool_class();
    static INT_CLASS: Rc<Class> = build_int_class();
    static STRING_CLASS: Rc<Class> = build_string_class();
    static ARRAY_CLASS: Rc<Class> = build_array_class();
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
pub fn array_class() -> Rc<Class> {
    ARRAY_CLASS.with(|c| c.clone())
}

pub fn native_module() -> Value {
    NATIVE_MODULE.with(|c| Value::Object(c.clone(), Rc::new(vec![])))
}
