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
            _ => Eval::Error(RuntimeError::PrimitiveTypeError {
                expected: "integer".to_string(),
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
    Object(Rc<Class>, Vec<Value>),
}

impl Value {
    pub fn send(&self, ctx: &mut Interpreter, selector: &str, args: Vec<Value>) -> Eval {
        match self {
            Self::Integer(val) => return int_class(ctx, selector, *val, &args),
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
