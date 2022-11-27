use crate::{
    interpreter::{Eval, Interpreter},
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Constant(Value),
    Local(usize),
    Assign(usize),
    Send(String, usize),
}

impl IR {
    pub fn eval(&self, ctx: &mut Interpreter) -> Eval {
        match self {
            IR::Constant(value) => ctx.push(value.clone()),
            IR::Assign(index) => ctx.assign(*index),
            IR::Local(index) => ctx.get_local(*index),
            IR::Send(selector, arity) => {
                return ctx.send(selector, *arity);
            }
        };
        Eval::Ok
    }
}
