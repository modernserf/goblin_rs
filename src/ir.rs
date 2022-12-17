use std::rc::Rc;

use crate::runtime::{Class, MoreFn, Runtime, Value};

pub type Address = usize;
pub type Selector = String;
pub type Index = usize;
pub type Arity = usize;
pub type NativeFn = fn(Value, Vec<Value>) -> Runtime<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Unit,                        // (-- value)
    Bool(bool),                  // (-- value)
    Integer(i64),                // (-- value)
    String(Rc<String>),          // (-- value)
    MutArray,                    // (-- array)
    Local(Address),              // ( -- *address)
    Var(Address),                // ( -- address)
    IVal(Index),                 // ( -- instance[index])
    SelfRef,                     // ( -- self_value)
    Module(String),              // ( -- module)
    Object(Rc<Class>, Arity),    // (...instance -- object)
    DoObject(Rc<Class>, Arity),  // (...instance -- object)
    NewSelf(Arity),              // (...instance -- object)
    Deref,                       // (address -- *address)
    SetVar,                      // (value address -- )
    Send(Selector, Arity),       // (...args target -- result)
    TrySend(Selector, Arity),    // (...args target -- result)
    SendNative(NativeFn, Arity), // (...args target -- result)
    #[allow(unused)]
    SendNativeMore(MoreFn), // (...)
    SendBool,                    // (target bool -- result)
    Drop,                        // (value --)
    Return,
    Loop,
}

impl IR {
    pub fn send(selector: &str, arity: usize) -> Self {
        Self::Send(selector.to_string(), arity)
    }
}
