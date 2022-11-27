use crate::{
    interpreter::{Eval, Interpreter},
    value::Value,
};

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Constant(Value),
}

impl IR {
    pub fn eval(&self, ctx: &mut Interpreter) -> Eval {
        match self {
            IR::Constant(value) => {
                ctx.push(value.clone());
            }
        };
        Eval::Ok
    }
}
