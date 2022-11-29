use std::rc::Rc;

use crate::{
    class::Class,
    interpreter::{Eval, Interpreter, RuntimeError},
};

fn does_not_understand(selector: &str) -> Eval {
    Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string()))
}

fn primitive_type_error(expected: &str, arg: &Value) -> Eval {
    Eval::Error(RuntimeError::PrimitiveTypeError {
        expected: expected.to_string(),
        received: arg.clone(),
    })
}

fn int_class(ctx: &mut Interpreter, selector: &str, target: i64, args: &[Value]) -> Eval {
    match selector {
        "-" => {
            ctx.push(Value::Integer(-target));
            Eval::Ok
        }
        "+:" => match args[0] {
            Value::Integer(r) => {
                ctx.push(Value::Integer(target + r));
                Eval::Ok
            }
            Value::Float(f) => {
                ctx.push(Value::Float(target as f64 + f));
                Eval::Ok
            }
            _ => primitive_type_error("number", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

fn float_class(ctx: &mut Interpreter, selector: &str, target: f64, args: &[Value]) -> Eval {
    match selector {
        "-" => {
            ctx.push(Value::Float(-target));
            Eval::Ok
        }
        "+:" => match args[0] {
            Value::Integer(r) => {
                ctx.push(Value::Float(target + r as f64));
                Eval::Ok
            }
            Value::Float(r) => {
                ctx.push(Value::Float(target + r));
                Eval::Ok
            }
            _ => primitive_type_error("number", &args[0]),
        },
        _ => Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string())),
    }
}

fn string_class(
    ctx: &mut Interpreter,
    selector: &str,
    target: &Rc<String>,
    args: &[Value],
) -> Eval {
    match selector {
        "++:" => match &args[0] {
            Value::String(arg) => {
                let concat = format!("{}{}", target, arg);
                ctx.push(Value::String(Rc::new(concat)));
                Eval::Ok
            }
            _ => primitive_type_error("string", &args[0]),
        },
        _ => does_not_understand(selector),
    }
}

#[derive(Debug)]
pub enum Handler {}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Integer(i64),
    Float(f64),
    String(Rc<String>),
    Object(Rc<Class>, Vec<Value>),
}

impl Value {
    pub fn send(&self, ctx: &mut Interpreter, selector: &str, args: Vec<Value>) -> Eval {
        match self {
            Self::Integer(target) => int_class(ctx, selector, *target, &args),
            Self::Float(target) => float_class(ctx, selector, *target, &args),
            Self::String(target) => string_class(ctx, selector, target, &args),
            Self::Object(cls, ivars) => {
                if let Some(handler) = cls.get(selector) {
                    return handler.send(ctx, args, ivars);
                }
                does_not_understand(selector)
            }
            _ => does_not_understand(selector),
        }
    }
}
