use std::ops::Deref;
use std::{cell::RefCell, rc::Rc, vec};

use crate::class::{Class, Object, Param, RcClass};
use crate::ir::IR;
use crate::runtime::RuntimeError;
use crate::value::Value;

fn build_true_class() -> RcClass {
    let mut class = Class::new();
    // match
    class.add_handler(":", vec![Param::Do], vec![IR::send("true", 0)]);

    // equality
    class.add_native("=:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(true)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(false)),
        _ => Ok(Value::bool(false)),
    });
    class.add_native("!=:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(false)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(true)),
        _ => Ok(Value::bool(true)),
    });
    // logical operators
    class.add_native("!", vec![], |_, _| Ok(Value::bool(false)));
    class.add_native("&&:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(true)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(false)),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native("||:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(true)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(true)),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native(
        "false:true:",
        vec![Param::Value, Param::Value],
        |_, args| Ok(args[1].clone()),
    );

    class.rc()
}

fn build_false_class() -> RcClass {
    let mut class = Class::new();
    // match
    class.add_handler(":", vec![Param::Do], vec![IR::send("false", 0)]);

    // equality
    class.add_native("=:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(false)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(true)),
        _ => Ok(Value::bool(false)),
    });
    class.add_native("!=:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(true)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(false)),
        _ => Ok(Value::bool(true)),
    });
    // logical operators
    class.add_native("!", vec![], |_, _| Ok(Value::bool(true)));
    class.add_native("&&:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(false)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(false)),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native("||:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::bool(true)),
        Value::Primitive(Primitive::False) => Ok(Value::bool(false)),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native(
        "false:true:",
        vec![Param::Value, Param::Value],
        |_, args| Ok(args[0].clone()),
    );

    class.rc()
}

fn build_int_class() -> RcClass {
    let mut class = Class::new();
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target.as_integer() == r)),
        _ => Ok(Value::bool(false)),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match args[0] {
        Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target.as_integer() != r)),
        _ => Ok(Value::bool(true)),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| {
        let str = target.as_integer().to_string();
        Ok(Value::Primitive(Primitive::String(Rc::new(str))))
    });
    // arithmetic
    class.add_native("-", vec![], |it, _| Ok(Value::int(-it.as_integer())));
    class.add_native("abs", vec![], |it, _| Ok(Value::int(it.as_integer().abs())));
    class.add_native("+:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target + r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::float(target as f64 + f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("-:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target - r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::float(target as f64 - f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("*:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target * r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::float(target as f64 * f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    // bitwise
    class.add_native("<<:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target << r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native(">>:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target >> r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("&:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target & r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("|:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target | r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("^:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target ^ r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    // minmax
    class.add_native("min:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target.min(r))),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::float((target as f64).min(f))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("max:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::int(target.max(r))),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::float((target as f64).max(f))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(
        "max:min:",
        vec![Param::Value, Param::Value],
        |target, args| {
            let target = target.as_integer();
            match (&args[0], &args[1]) {
                (
                    Value::Primitive(Primitive::Float(max)),
                    Value::Primitive(Primitive::Float(min)),
                ) => Ok(Value::float((target as f64).clamp(*min, *max))),
                (
                    Value::Primitive(Primitive::Float(max)),
                    Value::Primitive(Primitive::Integer(min)),
                ) => Ok(Value::float((target as f64).clamp(*min as f64, *max))),
                (
                    Value::Primitive(Primitive::Integer(max)),
                    Value::Primitive(Primitive::Float(min)),
                ) => Ok(Value::float((target as f64).clamp(*min, *max as f64))),
                (
                    Value::Primitive(Primitive::Integer(max)),
                    Value::Primitive(Primitive::Integer(min)),
                ) => Ok(Value::int(target.clamp(*min, *max))),
                _ => RuntimeError::primitive_type_error("number", &args[0]),
            }
        },
    );
    // comparison
    class.add_native("<:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target < r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool((target as f64) < f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<=:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target <= r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool((target as f64) <= f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("==:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target == r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool((target as f64) == f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<>:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target != r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool((target as f64) != f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">=:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target >= r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool((target as f64) >= f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target > r)),
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool((target as f64) > f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_handler(
        "order:",
        vec![Param::Value],
        vec![
            IR::SelfRef,
            IR::send("-:", 1),
            IR::Module("core".to_string()),
            IR::send("Ord", 0),
            IR::send("from int:", 1),
        ],
    );

    class.rc()
}

fn build_float_class() -> RcClass {
    let mut class = Class::new();
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target.as_float() == f)),
        _ => Ok(Value::bool(false)),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match args[0] {
        Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target.as_float() != f)),
        _ => Ok(Value::bool(true)),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| {
        let str = target.as_float().to_string();
        Ok(Value::Primitive(Primitive::String(Rc::new(str))))
    });
    // arithmetic
    class.add_native("-", vec![], |target, _| {
        Ok(Value::float(-target.as_float()))
    });
    class.add_native("abs", vec![], |target, _| {
        Ok(Value::float(target.as_float().abs()))
    });
    class.add_native("+:", vec![Param::Value], |target, args| match args[0] {
        Value::Primitive(Primitive::Integer(r)) => Ok(Value::float(target.as_float() + r as f64)),
        Value::Primitive(Primitive::Float(f)) => Ok(Value::float(target.as_float() + f)),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("-:", vec![Param::Value], |target, args| match args[0] {
        Value::Primitive(Primitive::Integer(r)) => Ok(Value::float(target.as_float() - r as f64)),
        Value::Primitive(Primitive::Float(f)) => Ok(Value::float(target.as_float() - f)),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("*:", vec![Param::Value], |target, args| match args[0] {
        Value::Primitive(Primitive::Integer(r)) => Ok(Value::float(target.as_float() * r as f64)),
        Value::Primitive(Primitive::Float(f)) => Ok(Value::float(target.as_float() * f)),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    // comparison
    class.add_native("<:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target < f)),
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target < (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<=:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target <= f)),
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target <= (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("==:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target == f)),
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target == (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<>:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target != f)),
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target != (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">=:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target >= f)),
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target >= (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Primitive(Primitive::Float(f)) => Ok(Value::bool(target > f)),
            Value::Primitive(Primitive::Integer(r)) => Ok(Value::bool(target > (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.rc()
}

pub fn build_string_class() -> RcClass {
    let mut class = Class::new();
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Primitive(Primitive::String(r)) => Ok(Value::bool(target.as_string() == r)),
        _ => Ok(Value::bool(false)),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Primitive(Primitive::String(r)) => Ok(Value::bool(target.as_string() != r)),
        _ => Ok(Value::bool(true)),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| {
        Ok(Value::Primitive(target))
    });
    // chars
    class.add_native("length", vec![], |target, _| {
        Ok(Value::int(target.as_string().len() as i64))
    });
    class.add_native("code at:", vec![Param::Value], |target, args| {
        match &args[0] {
            Value::Primitive(Primitive::Integer(idx)) => {
                let target = target.as_string();
                if target.is_empty() {
                    return Err(RuntimeError::IndexOutOfRange);
                }
                let idx_ = (*idx).rem_euclid(target.len() as i64) as usize;
                let ch = target.chars().nth(idx_).unwrap();
                Ok(Value::int(ch as i64))
            }
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("at:", vec![Param::Value], |target, args| match &args[0] {
        Value::Primitive(Primitive::Integer(idx)) => {
            let target = target.as_string();
            if target.is_empty() {
                return Ok(Value::string(""));
            }
            let idx_ = (*idx).rem_euclid(target.len() as i64) as usize;
            let str = target[idx_..idx_ + 1].to_string();
            Ok(Value::Primitive(Primitive::String(Rc::new(str))))
        }
        _ => RuntimeError::primitive_type_error("integer", &args[0]),
    });
    // slicing
    class.add_native(
        "from:to:",
        vec![Param::Value, Param::Value],
        |target, args| match (&args[0], &args[1]) {
            (
                Value::Primitive(Primitive::Integer(from)),
                Value::Primitive(Primitive::Integer(to)),
            ) => {
                let target = target.as_string();
                if target.is_empty() {
                    return Ok(Value::string(""));
                }

                // TODO: how, exactly, should slice work?
                let from = (*from).rem_euclid(target.len() as i64) as usize;
                let to = (*to) as usize; //.rem_euclid(target.len() as i64) as usize;
                let str = target[from..to].to_string();
                Ok(Value::Primitive(Primitive::String(Rc::new(str))))
            }
            (_, _) => RuntimeError::primitive_type_error("integer", &args[0]),
        },
    );
    class.add_handler(
        "from:",
        vec![Param::Value],
        vec![
            IR::SelfRef,
            IR::send("length", 0),
            IR::SelfRef,
            IR::send("from:to:", 2),
        ],
    );
    class.add_handler(
        "to:",
        vec![Param::Value],
        vec![
            IR::Constant(Value::int(0)),
            IR::Local { index: 0 },
            IR::SelfRef,
            IR::send("from:to:", 2),
        ],
    );
    // concatenation
    class.add_handler(
        "++:",
        vec![Param::Value],
        vec![
            IR::send("to String", 0),
            IR::SelfRef,
            IR::send_primitive(
                |target, args| match &args[0] {
                    Value::Primitive(Primitive::String(arg)) => {
                        let concat = format!("{}{}", target.as_string(), arg);
                        Ok(Value::Primitive(Primitive::String(Rc::new(concat))))
                    }
                    _ => RuntimeError::primitive_type_error("string", &args[0]),
                },
                1,
            ),
        ],
    );
    class.rc()
}

pub fn build_cell_class() -> RcClass {
    let mut class = Class::new();
    class.add_native("", vec![], |target, _| {
        Ok(target.as_cell().borrow().clone())
    });
    class.add_native(":", vec![Param::Value], |target, mut args| {
        let arg = std::mem::take(&mut args[0]);
        let mut tgt = target.as_cell().borrow_mut();
        *tgt = arg;
        Ok(Value::unit())
    });
    class.rc()
}

fn get_cell_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, mut args| {
        let arg = std::mem::take(&mut args[0]);
        Ok(Value::Primitive(Primitive::Cell(Rc::new(RefCell::new(
            arg,
        )))))
    });
    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_assert_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::unit()),
        Value::Primitive(Primitive::False) => {
            RuntimeError::assertion_error("expected false to be true")
        }
        _ => RuntimeError::primitive_type_error("bool", &args[0].clone()),
    });
    class.add_native("true:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => Ok(Value::unit()),
        Value::Primitive(Primitive::False) => {
            RuntimeError::assertion_error("expected false to be true")
        }
        _ => RuntimeError::primitive_type_error("bool", &args[0].clone()),
    });
    class.add_native("false:", vec![Param::Value], |_, args| match &args[0] {
        Value::Primitive(Primitive::True) => {
            RuntimeError::assertion_error("expected true to be false")
        }
        Value::Primitive(Primitive::False) => Ok(Value::unit()),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native(
        "expected:received:",
        vec![Param::Value, Param::Value],
        |_, args| {
            if args[0] == args[1] {
                return Ok(Value::unit());
            }
            return RuntimeError::assertion_error(&format!(
                "expected: {:?}\nreceived: {:?}",
                args[0], args[1]
            ));
        },
    );
    class.add_handler(
        "panics:",
        vec![Param::Value],
        vec![
            IR::Spawn,
            IR::send_primitive(
                |target, _| {
                    if target.as_bool() {
                        return RuntimeError::assertion_error("expected handler to panic");
                    } else {
                        return Ok(Value::unit());
                    }
                },
                0,
            ),
        ],
    );

    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_file_module() -> Value {
    use std::fs;
    let mut class = Class::new();
    class.add_native(
        "read text sync:",
        vec![Param::Value],
        |_, args| match &args[0] {
            Value::Primitive(Primitive::String(path)) => match fs::read_to_string(path.deref()) {
                Ok(text) => Ok(Value::Primitive(Primitive::String(Rc::new(text)))),
                _ => todo!("error loading file"),
            },
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        },
    );

    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_string_module() -> Value {
    let mut class = Class::new();
    class.add_constant("newline", Value::string("\n"));
    class.add_constant("tab", Value::string("\t"));
    class.add_native(
        "from char code:",
        vec![Param::Value],
        |_, args| match args[0] {
            Value::Primitive(Primitive::Integer(d)) => match char::from_u32(d as u32) {
                Some(ch) => Ok(Value::Primitive(Primitive::String(Rc::new({
                    let mut s = String::new();
                    s.push(ch);
                    s
                })))),
                None => todo!("invalid char code"),
            },
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        },
    );

    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_panic_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, args| {
        RuntimeError::panic(&args[0])
    });

    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_log_module() -> Value {
    let mut class = Class::new();
    class.add_handler(
        ":",
        vec![Param::Value],
        vec![
            IR::send("to String", 0),
            IR::send_primitive(
                |target, _| {
                    println!("{}", target.as_string());
                    Ok(Value::unit())
                },
                0,
            ),
        ],
    );

    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_loop_module() -> Value {
    let mut class = Class::new();
    class.add_handler(
        ":",
        vec![Param::Do],
        vec![IR::Local { index: 0 }, IR::send("", 0), IR::Loop],
    );

    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_native_module() -> RcClass {
    let mut class = Class::new();
    class.add_constant("true", Value::bool(true));
    class.add_constant("false", Value::bool(false));
    class.add_constant("Cell", get_cell_module());
    class.add_constant("Assert", get_assert_module());
    class.add_constant("File", get_file_module());
    class.add_constant("String", get_string_module());
    class.add_constant("Panic", get_panic_module());
    class.add_constant("Log", get_log_module());
    class.add_constant("loop", get_loop_module());

    class.rc()
}

thread_local! {
    static TRUE_CLASS : RcClass = build_true_class();
    static FALSE_CLASS : RcClass = build_false_class();
    static INT_CLASS : RcClass = build_int_class();
    static FLOAT_CLASS : RcClass = build_float_class();
    static STRING_CLASS : RcClass = build_string_class();
    static CELL_CLASS : RcClass = build_cell_class();

    static NATIVE_MODULE : RcClass = get_native_module()
}

pub fn native_module() -> Value {
    NATIVE_MODULE.with(|x| Value::Object(Rc::new(Object::new(x.clone(), vec![]))))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Unit,
    True,
    False,
    Integer(i64),
    Float(f64),
    String(Rc<String>),
    Cell(Rc<RefCell<Value>>),
}

impl Primitive {
    pub fn as_bool(&self) -> bool {
        match self {
            Self::True => true,
            Self::False => false,
            _ => panic!("expected bool"),
        }
    }

    pub fn as_integer(&self) -> i64 {
        match self {
            Self::Integer(val) => *val,
            _ => panic!("expected integer"),
        }
    }
    pub fn as_float(&self) -> f64 {
        match self {
            Self::Float(val) => *val,
            _ => panic!("expected float"),
        }
    }
    pub fn as_string(&self) -> &Rc<String> {
        match self {
            Self::String(str) => str,
            _ => panic!("expected string"),
        }
    }
    pub fn as_cell(&self) -> &Rc<RefCell<Value>> {
        match self {
            Self::Cell(cell) => cell,
            _ => panic!("expected cell"),
        }
    }

    pub fn class(&self) -> RcClass {
        match self {
            Self::Unit => Class::new().rc(),
            Self::True => TRUE_CLASS.with(|c| c.clone()),
            Self::False => FALSE_CLASS.with(|c| c.clone()),
            Self::Integer(..) => INT_CLASS.with(|c| c.clone()),
            Self::Float(..) => FLOAT_CLASS.with(|c| c.clone()),
            Self::String(..) => STRING_CLASS.with(|c| c.clone()),
            Self::Cell(..) => CELL_CLASS.with(|c| c.clone()),
        }
    }
}
