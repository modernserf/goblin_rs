use crate::{class::RcClass, value::Value};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Drop,
    Return,
    Constant(Value),
    Local(usize),
    Assign(usize),
    Send(String, usize),
    TrySend(String, usize),
    Object(RcClass, usize),
    SelfObject(usize),
    IVar(usize),
    VarArg(usize),
    SelfRef,
    DoBlock { class: RcClass, own_offset: usize },
    Allocate(usize),
    Debug(String),
}
