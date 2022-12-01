use crate::{class::RcClass, value::Value};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Drop,
    Constant(Value),
    Local(usize),
    Assign(usize),
    Send(String, usize),
    Object(RcClass, usize),
    SelfObject(usize),
    IVar(usize),
    SelfRef,
    DoBlock { class: RcClass, own_offset: usize },
    Allocate(usize),
    Debug(String),
}
