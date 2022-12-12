use crate::{class::RcClass, primitive::Primitive, runtime::Runtime, value::Value};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    // put a value on the stack
    SelfRef,
    Unit,
    Constant(Value),
    Module(String),
    Local { index: usize },
    IVar { index: usize },
    Parent { index: usize },
    VarArg { index: usize },
    // consume stack values
    Drop,
    SetLocal { index: usize },
    SetParent { index: usize },
    Send { selector: String, arity: usize },
    SendPrimitive { f: NativeHandlerFn, arity: usize },
    TrySend { selector: String, arity: usize },
    NewObject { class: RcClass, arity: usize },
    NewDoObject { class: RcClass },
    NewSelf { arity: usize },
    Spawn,
    // control flow
    Return,
    Loop,
}

impl IR {
    #[cfg(test)]
    pub fn int(value: i64) -> IR {
        IR::Constant(Value::int(value))
    }
    pub fn send(selector: &str, arity: usize) -> IR {
        IR::Send {
            selector: selector.to_string(),
            arity,
        }
    }
    pub fn send_primitive(f: NativeHandlerFn, arity: usize) -> IR {
        IR::SendPrimitive { f, arity }
    }
    #[cfg(test)]
    pub fn new_object(class: &RcClass, arity: usize) -> IR {
        IR::NewObject {
            class: class.clone(),
            arity,
        }
    }
}

pub type NativeHandlerFn = fn(Primitive, Vec<Value>) -> Runtime<Value>;
