use crate::interpreter::{Eval, Interpreter, RuntimeError};

// pub struct Class {
//     handlers: HashMap<String, Handler>,
//     else_handler: Option<Handler>,
// }

// impl Class {
//     pub fn new() -> Self {
//         Class {
//             handlers: HashMap::new(),
//             else_handler: None,
//         }
//     }
// }

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
}

impl Value {
    pub fn send(&self, ctx: &mut Interpreter, selector: &str, args: &[Value]) -> Eval {
        match self {
            Value::Integer(val) => int_class(ctx, selector, *val, args),
            _ => Eval::Error(RuntimeError::DoesNotUnderstand(selector.to_string())),
        }
    }
}
