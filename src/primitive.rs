use std::{cell::RefCell, ops::Deref, rc::Rc, vec};

use crate::class::{Class, Object, Param, RcClass};
use crate::runtime::{RuntimeError, IR};
use crate::value::Value;

fn build_true_class() -> RcClass {
    let mut class = Class::new();
    // match
    class.add_handler(":", vec![Param::Do], vec![IR::send("true", 0)]);

    // equality
    class.add_native("=:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::True),
        Value::False => Ok(Value::False),
        _ => Ok(Value::False),
    });
    class.add_native("!=:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::False),
        Value::False => Ok(Value::True),
        _ => Ok(Value::True),
    });
    // logical operators
    class.add_native("!", vec![], |_, _| Ok(Value::False));
    class.add_native("&&:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::True),
        Value::False => Ok(Value::False),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native("||:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::True),
        Value::False => Ok(Value::True),
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
        Value::True => Ok(Value::False),
        Value::False => Ok(Value::True),
        _ => Ok(Value::False),
    });
    class.add_native("!=:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::True),
        Value::False => Ok(Value::False),
        _ => Ok(Value::True),
    });
    // logical operators
    class.add_native("!", vec![], |_, _| Ok(Value::True));
    class.add_native("&&:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::False),
        Value::False => Ok(Value::False),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native("||:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::True),
        Value::False => Ok(Value::False),
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
        Value::Integer(r) => Ok(Value::bool(target.as_integer() == r)),
        _ => Ok(Value::False),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Ok(Value::bool(target.as_integer() != r)),
        _ => Ok(Value::True),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| {
        let str = target.as_integer().to_string();
        Ok(Value::String(Rc::new(str)))
    });
    // arithmetic
    class.add_native("-", vec![], |it, _| Ok(Value::Integer(-it.as_integer())));
    class.add_native("abs", vec![], |it, _| {
        Ok(Value::Integer(it.as_integer().abs()))
    });
    class.add_native("+:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target + r)),
            Value::Float(f) => Ok(Value::Float(target as f64 + f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("-:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target - r)),
            Value::Float(f) => Ok(Value::Float(target as f64 - f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("*:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target * r)),
            Value::Float(f) => Ok(Value::Float(target as f64 * f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    // bitwise
    class.add_native("<<:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target << r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native(">>:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target >> r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("&:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target & r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("|:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target | r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("^:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target ^ r)),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    // minmax
    class.add_native("min:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target.min(r))),
            Value::Float(f) => Ok(Value::Float((target as f64).min(f))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("max:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::Integer(target.max(r))),
            Value::Float(f) => Ok(Value::Float((target as f64).max(f))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(
        "max:min:",
        vec![Param::Value, Param::Value],
        |target, args| {
            let target = target.as_integer();
            match (&args[0], &args[1]) {
                (Value::Float(max), Value::Float(min)) => {
                    Ok(Value::Float((target as f64).clamp(*min, *max)))
                }
                (Value::Float(max), Value::Integer(min)) => {
                    Ok(Value::Float((target as f64).clamp(*min as f64, *max)))
                }
                (Value::Integer(max), Value::Float(min)) => {
                    Ok(Value::Float((target as f64).clamp(*min, *max as f64)))
                }
                (Value::Integer(max), Value::Integer(min)) => {
                    Ok(Value::Integer(target.clamp(*min, *max)))
                }
                _ => RuntimeError::primitive_type_error("number", &args[0]),
            }
        },
    );
    // comparison
    class.add_native("<:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::bool(target < r)),
            Value::Float(f) => Ok(Value::bool((target as f64) < f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<=:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::bool(target <= r)),
            Value::Float(f) => Ok(Value::bool((target as f64) <= f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("==:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::bool(target == r)),
            Value::Float(f) => Ok(Value::bool((target as f64) == f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<>:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::bool(target != r)),
            Value::Float(f) => Ok(Value::bool((target as f64) != f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">=:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::bool(target >= r)),
            Value::Float(f) => Ok(Value::bool((target as f64) >= f)),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Ok(Value::bool(target > r)),
            Value::Float(f) => Ok(Value::bool((target as f64) > f)),
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
        Value::Float(r) => Ok(Value::bool(target.as_float() == r)),
        _ => Ok(Value::False),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match args[0] {
        Value::Float(r) => Ok(Value::bool(target.as_float() != r)),
        _ => Ok(Value::True),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| {
        let str = target.as_float().to_string();
        Ok(Value::String(Rc::new(str)))
    });
    // arithmetic
    class.add_native("-", vec![], |target, _| {
        Ok(Value::Float(-target.as_float()))
    });
    class.add_native("abs", vec![], |target, _| {
        Ok(Value::Float(target.as_float().abs()))
    });
    class.add_native("+:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Ok(Value::Float(target.as_float() + r as f64)),
        Value::Float(r) => Ok(Value::Float(target.as_float() + r)),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("-:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Ok(Value::Float(target.as_float() - r as f64)),
        Value::Float(r) => Ok(Value::Float(target.as_float() - r)),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    class.add_native("*:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Ok(Value::Float(target.as_float() * r as f64)),
        Value::Float(r) => Ok(Value::Float(target.as_float() * r)),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    // comparison
    class.add_native("<:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Ok(Value::bool(target < f)),
            Value::Integer(r) => Ok(Value::bool(target < (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<=:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Ok(Value::bool(target <= f)),
            Value::Integer(r) => Ok(Value::bool(target <= (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("==:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Ok(Value::bool(target == f)),
            Value::Integer(r) => Ok(Value::bool(target == (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<>:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Ok(Value::bool(target != f)),
            Value::Integer(r) => Ok(Value::bool(target != (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">=:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Ok(Value::bool(target >= f)),
            Value::Integer(r) => Ok(Value::bool(target >= (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Ok(Value::bool(target > f)),
            Value::Integer(r) => Ok(Value::bool(target > (r as f64))),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.rc()
}

pub fn build_string_class() -> RcClass {
    let mut class = Class::new();
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(r) => Ok(Value::bool(target.as_string() == r)),
        _ => Ok(Value::False),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(r) => Ok(Value::bool(target.as_string() != r)),
        _ => Ok(Value::True),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| Ok(target));
    // chars
    class.add_native("length", vec![], |target, _| {
        Ok(Value::Integer(target.as_string().len() as i64))
    });
    class.add_native("code at:", vec![Param::Value], |target, args| {
        match &args[0] {
            Value::Integer(idx) => {
                let target = target.as_string();
                if target.is_empty() {
                    return Err(RuntimeError::IndexOutOfRange);
                }
                let idx_ = (*idx).rem_euclid(target.len() as i64) as usize;
                let ch = target.chars().nth(idx_).unwrap();
                Ok(Value::Integer(ch as i64))
            }
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("at:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(idx) => {
            let target = target.as_string();
            if target.is_empty() {
                return Ok(Value::string(""));
            }
            let idx_ = (*idx).rem_euclid(target.len() as i64) as usize;
            let str = target[idx_..idx_ + 1].to_string();
            Ok(Value::String(Rc::new(str)))
        }
        _ => RuntimeError::primitive_type_error("integer", &args[0]),
    });
    // slicing
    class.add_native(
        "from:to:",
        vec![Param::Value, Param::Value],
        |target, args| match (&args[0], &args[1]) {
            (Value::Integer(from), Value::Integer(to)) => {
                let target = target.as_string();
                if target.is_empty() {
                    return Ok(Value::string(""));
                }

                // TODO: how, exactly, should slice work?
                let from = (*from).rem_euclid(target.len() as i64) as usize;
                let to = (*to) as usize; //.rem_euclid(target.len() as i64) as usize;
                let str = target[from..to].to_string();
                Ok(Value::String(Rc::new(str)))
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
            IR::Constant(Value::Integer(0)),
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
                    Value::String(arg) => {
                        let concat = format!("{}{}", target.as_string(), arg);
                        Ok(Value::String(Rc::new(concat)))
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
        Ok(target.as_cell().deref().borrow().clone())
    });
    class.add_native(":", vec![Param::Value], |target, mut args| {
        let arg = std::mem::take(&mut args[0]);
        let mut tgt = target.as_cell().borrow_mut();
        *tgt = arg;
        Ok(Value::Unit)
    });
    class.rc()
}

fn get_cell_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, mut args| {
        let arg = std::mem::take(&mut args[0]);
        Ok(Value::Cell(Rc::new(RefCell::new(arg))))
    });
    let obj = Object::new(class.rc(), vec![]);
    Value::Object(Rc::new(obj))
}

fn get_assert_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::Unit),
        Value::False => RuntimeError::assertion_error("expected false to be true"),
        _ => RuntimeError::primitive_type_error("bool", &args[0].clone()),
    });
    class.add_native("true:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => Ok(Value::Unit),
        Value::False => RuntimeError::assertion_error("expected false to be true"),
        _ => RuntimeError::primitive_type_error("bool", &args[0].clone()),
    });
    class.add_native("false:", vec![Param::Value], |_, args| match &args[0] {
        Value::True => RuntimeError::assertion_error("expected true to be false"),
        Value::False => Ok(Value::Unit),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native(
        "expected:received:",
        vec![Param::Value, Param::Value],
        |_, args| {
            if args[0] == args[1] {
                return Ok(Value::Unit);
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
                        return Ok(Value::Unit);
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
            Value::String(path) => match fs::read_to_string(path.deref()) {
                Ok(text) => Ok(Value::String(Rc::new(text))),
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
    class.add_constant("newline", Value::String(Rc::new("\n".to_string())));
    class.add_constant("tab", Value::String(Rc::new("\t".to_string())));
    class.add_native(
        "from char code:",
        vec![Param::Value],
        |_, args| match args[0] {
            Value::Integer(d) => match char::from_u32(d as u32) {
                Some(ch) => Ok(Value::String(Rc::new({
                    let mut s = String::new();
                    s.push(ch);
                    s
                }))),
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
                    Ok(Value::Unit)
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
    class.add_constant("true", Value::True);
    class.add_constant("false", Value::False);
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
pub fn true_class() -> RcClass {
    TRUE_CLASS.with(|c| c.clone())
}
pub fn false_class() -> RcClass {
    FALSE_CLASS.with(|c| c.clone())
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
