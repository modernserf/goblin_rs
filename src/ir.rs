use crate::{class::RcClass, interpreter::SendEffect, value::Value};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Spawn,
    Drop,
    Return,
    Constant(Value),
    Local(usize),
    Assign(usize),
    Send(String, usize),
    SendPrimitive(NativeHandlerFn, usize),
    TrySend(String, usize),
    Object(RcClass, usize),
    SelfObject(usize),
    IVar(usize),
    VarArg(usize),
    SelfRef,
    DoBlock { class: RcClass, own_offset: usize },
    Allocate(usize),
    Debug(String),
    Module(String),
}

pub type NativeHandlerFn = fn(Value, Vec<Value>) -> SendEffect;
