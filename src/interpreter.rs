use crate::{ir::IR, value::Value};

#[derive(Debug)]
pub struct Interpreter {
    stack: Vec<Value>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeError {
    DoesNotUnderstand(String),
    PrimitiveTypeError { expected: String, received: Value },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Eval {
    Ok,
    Error(RuntimeError),
}

impl Interpreter {
    pub fn program(program: Vec<IR>) -> Result<Value, RuntimeError> {
        let mut ctx = Interpreter { stack: Vec::new() };
        for stmt in program.iter() {
            match stmt.eval(&mut ctx) {
                Eval::Ok => continue,
                Eval::Error(err) => return Err(err),
            }
        }
        ctx.stack.pop().map(Ok).unwrap_or(Ok(Value::Unit))
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }
    // TODO: convert frame-relative address to absolute address
    fn local(&self, index: usize) -> usize {
        index
    }
    pub fn get_local(&mut self, index: usize) {
        let idx = self.local(index);
        let value = self.stack[idx].clone();
        self.stack.push(value);
    }
    pub fn assign(&mut self, index: usize) {
        let idx = self.local(index);
        let top = self.stack.pop().unwrap();
        if idx == self.stack.len() {
            self.stack.push(top);
        } else {
            self.stack[idx] = top;
        }
    }
    pub fn send(&mut self, selector: &str, arity: usize) -> Eval {
        let args = self.stack.split_off(self.stack.len() - arity);
        let target = self.stack.pop().unwrap();
        target.send(self, selector, &args)
    }
}
