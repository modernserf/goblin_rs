use std::rc::Rc;

use crate::{
    class::Class,
    interpreter::{Eval, Interpreter, RuntimeError},
};

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
            _ => Eval::Error(RuntimeError::PrimitiveTypeError {
                expected: "number".to_string(),
                received: args[0].clone(),
            }),
        },
        _ => Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string())),
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
            _ => Eval::Error(RuntimeError::PrimitiveTypeError {
                expected: "number".to_string(),
                received: args[0].clone(),
            }),
        },
        _ => Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string())),
    }
}

#[derive(Debug)]
pub enum Handler {}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Integer(i64),
    Float(f64),
    Object(Rc<Class>, Vec<Value>),
}

impl Value {
    pub fn send(&self, ctx: &mut Interpreter, selector: &str, args: Vec<Value>) -> Eval {
        match self {
            Self::Integer(target) => return int_class(ctx, selector, *target, &args),
            Self::Float(target) => return float_class(ctx, selector, *target, &args),
            Self::Object(cls, ivars) => {
                if let Some(handler) = cls.get(selector) {
                    return handler.send(ctx, args, ivars);
                }
            }
            _ => (),
        };
        Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string()))
    }
}
