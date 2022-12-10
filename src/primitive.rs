use std::{cell::RefCell, ops::Deref, rc::Rc, vec};

use crate::{
    class::{Class, Object, Param, RcClass},
    ir::IR,
    runtime_error::RuntimeError,
    value::Value,
};

fn build_bool_class() -> RcClass {
    let mut class = Class::new();
    // match
    class.add_native(":", vec![Param::Do], |target, args| {
        let selector = if target.as_bool() { "true" } else { "false" };
        args[0].send(selector, vec![])
    });
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(other) => Value::Bool(target.as_bool() == *other).eval(),
        _ => Value::Bool(false).eval(),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(other) => Value::Bool(target.as_bool() != *other).eval(),
        _ => Value::Bool(true).eval(),
    });
    // logical operators
    class.add_native("!", vec![], |target, _| {
        Value::Bool(!target.as_bool()).eval()
    });
    class.add_native("&&:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(other) => Value::Bool(target.as_bool() && *other).eval(),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native("||:", vec![Param::Value], |target, args| match &args[0] {
        Value::Bool(other) => Value::Bool(target.as_bool() || *other).eval(),
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native(
        "false:true:",
        vec![Param::Value, Param::Value],
        |target, args| {
            if target.as_bool() {
                args[1].clone().eval()
            } else {
                args[0].clone().eval()
            }
        },
    );

    class.rc()
}

fn build_int_class() -> RcClass {
    let mut class = Class::new();
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Bool(target.as_integer() == r).eval(),
        _ => Value::Bool(false).eval(),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Bool(target.as_integer() != r).eval(),
        _ => Value::Bool(true).eval(),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| {
        let str = target.as_integer().to_string();
        Value::String(Rc::new(str)).eval()
    });
    // arithmetic
    class.add_native("-", vec![], |it, _| Value::Integer(-it.as_integer()).eval());
    class.add_native("abs", vec![], |it, _| {
        Value::Integer(it.as_integer().abs()).eval()
    });
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
    class.add_native("*:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target * r).eval(),
            Value::Float(f) => Value::Float(target as f64 * f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    // bitwise
    class.add_native("<<:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target << r).eval(),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native(">>:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target >> r).eval(),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("&:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target & r).eval(),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("|:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target | r).eval(),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("^:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target ^ r).eval(),
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    // minmax
    class.add_native("min:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target.min(r)).eval(),
            Value::Float(f) => Value::Float((target as f64).min(f)).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("max:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Integer(target.max(r)).eval(),
            Value::Float(f) => Value::Float((target as f64).max(f)).eval(),
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
                    Value::Float((target as f64).clamp(*min, *max)).eval()
                }
                (Value::Float(max), Value::Integer(min)) => {
                    Value::Float((target as f64).clamp(*min as f64, *max)).eval()
                }
                (Value::Integer(max), Value::Float(min)) => {
                    Value::Float((target as f64).clamp(*min, *max as f64)).eval()
                }
                (Value::Integer(max), Value::Integer(min)) => {
                    Value::Integer(target.clamp(*min, *max)).eval()
                }
                _ => RuntimeError::primitive_type_error("number", &args[0]),
            }
        },
    );
    // comparison
    class.add_native("<:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Bool(target < r).eval(),
            Value::Float(f) => Value::Bool((target as f64) < f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<=:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Bool(target <= r).eval(),
            Value::Float(f) => Value::Bool((target as f64) <= f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("==:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Bool(target == r).eval(),
            Value::Float(f) => Value::Bool((target as f64) == f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<>:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Bool(target != r).eval(),
            Value::Float(f) => Value::Bool((target as f64) != f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">=:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Bool(target >= r).eval(),
            Value::Float(f) => Value::Bool((target as f64) >= f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">:", vec![Param::Value], |target, args| {
        let target = target.as_integer();
        match args[0] {
            Value::Integer(r) => Value::Bool(target > r).eval(),
            Value::Float(f) => Value::Bool((target as f64) > f).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_handler(
        "order:",
        vec![Param::Value],
        /*
            on {order: other}
                import [_Ord_] := "core/ord"
                (it < other){
                    true: Ord{<}
                    false: (it > other){
                        true: Ord{>}
                        false: Ord{==}
                    }
                }
        */
        vec![
            IR::Module("core/ord".to_string()),
            IR::Send("Ord".to_string(), 0),
            IR::Assign(1),
            IR::IVar(0),
            IR::Local(0),
            IR::Send("<:".to_string(), 1),
            IR::IVar(0),
            IR::Local(0),
            IR::Send(">:".to_string(), 1),
            IR::Local(1),
            IR::Send("==".to_string(), 0),
            IR::Local(1),
            IR::Send(">".to_string(), 0),
            IR::Send("false:true:".to_string(), 2),
            IR::Local(1),
            IR::Send("<".to_string(), 0),
            IR::Send("false:true:".to_string(), 2),
        ],
    );

    class.rc()
}

fn build_float_class() -> RcClass {
    let mut class = Class::new();
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match args[0] {
        Value::Float(r) => Value::Bool(target.as_float() == r).eval(),
        _ => Value::Bool(false).eval(),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match args[0] {
        Value::Float(r) => Value::Bool(target.as_float() != r).eval(),
        _ => Value::Bool(true).eval(),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| {
        let str = target.as_float().to_string();
        Value::String(Rc::new(str)).eval()
    });
    // arithmetic
    class.add_native("-", vec![], |target, _| {
        Value::Float(-target.as_float()).eval()
    });
    class.add_native("abs", vec![], |target, _| {
        Value::Float(target.as_float().abs()).eval()
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
    class.add_native("*:", vec![Param::Value], |target, args| match args[0] {
        Value::Integer(r) => Value::Float(target.as_float() * r as f64).eval(),
        Value::Float(r) => Value::Float(target.as_float() * r).eval(),
        _ => RuntimeError::primitive_type_error("number", &args[0]),
    });
    // comparison
    class.add_native("<:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Value::Bool(target < f).eval(),
            Value::Integer(r) => Value::Bool(target < (r as f64)).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<=:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Value::Bool(target <= f).eval(),
            Value::Integer(r) => Value::Bool(target <= (r as f64)).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("==:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Value::Bool(target == f).eval(),
            Value::Integer(r) => Value::Bool(target == (r as f64)).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native("<>:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Value::Bool(target != f).eval(),
            Value::Integer(r) => Value::Bool(target != (r as f64)).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">=:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Value::Bool(target >= f).eval(),
            Value::Integer(r) => Value::Bool(target >= (r as f64)).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.add_native(">:", vec![Param::Value], |target, args| {
        let target = target.as_float();
        match args[0] {
            Value::Float(f) => Value::Bool(target > f).eval(),
            Value::Integer(r) => Value::Bool(target > (r as f64)).eval(),
            _ => RuntimeError::primitive_type_error("number", &args[0]),
        }
    });
    class.rc()
}

pub fn build_string_class() -> RcClass {
    let mut class = Class::new();
    // equality
    class.add_native("=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(r) => Value::Bool(target.as_string() == r).eval(),
        _ => Value::Bool(false).eval(),
    });
    class.add_native("!=:", vec![Param::Value], |target, args| match &args[0] {
        Value::String(r) => Value::Bool(target.as_string() != r).eval(),
        _ => Value::Bool(true).eval(),
    });
    // conversions
    class.add_native("to String", vec![], |target, _| target.eval());
    // chars
    class.add_native("length", vec![], |target, _| {
        Value::Integer(target.as_string().len() as i64).eval()
    });
    class.add_native("code at:", vec![Param::Value], |target, args| {
        match &args[0] {
            Value::Integer(idx) => {
                let target = target.as_string();
                if target.is_empty() {
                    return RuntimeError::index_out_of_range();
                }
                let idx_ = (*idx).rem_euclid(target.len() as i64) as usize;
                let ch = target.chars().nth(idx_).unwrap();
                Value::Integer(ch as i64).eval()
            }
            _ => RuntimeError::primitive_type_error("integer", &args[0]),
        }
    });
    class.add_native("at:", vec![Param::Value], |target, args| match &args[0] {
        Value::Integer(idx) => {
            let target = target.as_string();
            if target.is_empty() {
                return Value::string("").eval();
            }
            let idx_ = (*idx).rem_euclid(target.len() as i64) as usize;
            let str = target[idx_..idx_ + 1].to_string();
            Value::String(Rc::new(str)).eval()
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
                    return Value::string("").eval();
                }

                // TODO: how, exactly, should slice work?
                let from = (*from).rem_euclid(target.len() as i64) as usize;
                let to = (*to) as usize; //.rem_euclid(target.len() as i64) as usize;
                let str = target[from..to].to_string();
                Value::String(Rc::new(str)).eval()
            }
            (_, _) => RuntimeError::primitive_type_error("integer", &args[0]),
        },
    );
    class.add_handler(
        "from:",
        vec![Param::Value],
        vec![
            IR::IVar(0),
            IR::Local(0),
            IR::IVar(0),
            IR::Send("length".to_string(), 0),
            IR::Send("from:to:".to_string(), 2),
        ],
    );
    class.add_handler(
        "to:",
        vec![Param::Value],
        vec![
            IR::IVar(0),
            IR::Constant(Value::Integer(0)),
            IR::Local(0),
            IR::Send("from:to:".to_string(), 2),
        ],
    );
    // concatenation
    class.add_handler(
        "++:",
        vec![Param::Value],
        vec![
            IR::IVar(0),
            IR::Local(0),
            IR::Send("to String".to_string(), 0),
            IR::SendPrimitive(
                |target, args| match &args[0] {
                    Value::String(arg) => {
                        let concat = format!("{}{}", target.as_string(), arg);
                        Value::String(Rc::new(concat)).eval()
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

fn get_assert_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, args| match &args[0] {
        Value::Bool(value) => {
            if *value {
                Value::Unit.eval()
            } else {
                RuntimeError::assertion_error("expected false to be true")
            }
        }
        _ => RuntimeError::primitive_type_error("bool", &args[0].clone()),
    });
    class.add_native("true:", vec![Param::Value], |_, args| match &args[0] {
        Value::Bool(value) => {
            if *value {
                Value::Unit.eval()
            } else {
                RuntimeError::assertion_error("expected false to be true")
            }
        }
        _ => RuntimeError::primitive_type_error("bool", &args[0].clone()),
    });
    class.add_native("false:", vec![Param::Value], |_, args| match &args[0] {
        Value::Bool(value) => {
            if !value {
                Value::Unit.eval()
            } else {
                RuntimeError::assertion_error("expected false to be true")
            }
        }
        _ => RuntimeError::primitive_type_error("bool", &args[0]),
    });
    class.add_native(
        "expected:received:",
        vec![Param::Value, Param::Value],
        |_, args| {
            if args[0] == args[1] {
                return Value::Unit.eval();
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
            IR::Local(0),
            IR::Spawn,
            IR::SendPrimitive(
                |_, args| {
                    if args[0].as_bool() {
                        return RuntimeError::assertion_error("expected handler to panic");
                    } else {
                        return Value::Unit.eval();
                    }
                },
                1,
            ),
        ],
    );

    let obj = Object::new(class.rc(), vec![Value::Unit]);
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
                Ok(text) => Value::String(Rc::new(text)).eval(),
                _ => todo!("error loading file"),
            },
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        },
    );

    let obj = Object::new(class.rc(), vec![Value::Unit]);
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
                Some(ch) => Value::String(Rc::new({
                    let mut s = String::new();
                    s.push(ch);
                    s
                }))
                .eval(),
                None => todo!("invalid char code"),
            },
            _ => RuntimeError::primitive_type_error("string", &args[0]),
        },
    );

    let obj = Object::new(class.rc(), vec![Value::Unit]);
    Value::Object(Rc::new(obj))
}

fn get_panic_module() -> Value {
    let mut class = Class::new();
    class.add_native(":", vec![Param::Value], |_, args| {
        RuntimeError::panic(&args[0])
    });

    let obj = Object::new(class.rc(), vec![Value::Unit]);
    Value::Object(Rc::new(obj))
}

fn get_native_module() -> RcClass {
    let mut class = Class::new();
    class.add_constant("true", Value::Bool(true));
    class.add_constant("false", Value::Bool(false));
    class.add_constant("Cell", get_cell_module());
    class.add_constant("Assert", get_assert_module());
    class.add_constant("File", get_file_module());
    class.add_constant("String", get_string_module());
    class.add_constant("Panic", get_panic_module());

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
