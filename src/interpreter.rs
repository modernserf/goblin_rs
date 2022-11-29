use std::rc::Rc;

use crate::{
    class::{Body, Class},
    ir::IR,
    value::{IVars, Value},
};

#[derive(Debug)]
pub struct Interpreter {
    stack: Vec<Value>,
    frames: Vec<StackFrame>,
}

#[derive(Debug)]
pub struct StackFrame {
    class: Rc<Class>,
    ivars: IVars,
    offset: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeError {
    DoesNotUnderstand(String),
    PrimitiveTypeError { expected: String, received: Value },
}

#[derive(Debug, Clone)]
pub enum Eval {
    Ok,
    Error(RuntimeError),
    Call {
        class: Rc<Class>,
        args: Vec<Value>,
        ivars: IVars,
        body: Body,
    },
}

impl Interpreter {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: vec![StackFrame {
                class: Rc::new(Class::new()),
                ivars: Rc::new(Vec::new()),
                offset: 0,
            }],
        }
    }
    pub fn program(program: Vec<IR>) -> Result<Value, RuntimeError> {
        let mut ctx = Self::new();
        let mut call_stack = vec![(0 as usize, Rc::new(program))];
        loop {
            if let Some((i, body)) = call_stack.last_mut() {
                if *i < body.len() {
                    let stmt = &body[*i];
                    match stmt.eval(&mut ctx) {
                        Eval::Ok => {
                            *i += 1;
                            continue;
                        }
                        Eval::Error(err) => return Err(err),
                        Eval::Call {
                            class,
                            mut args,
                            ivars,
                            body,
                        } => {
                            *i += 1;
                            let body = body.clone();
                            ctx.push_frame(class, ivars);
                            ctx.stack.append(&mut args);
                            call_stack.push((0, body));
                            continue;
                        }
                    }
                }
                ctx.pop_frame();
                call_stack.pop();
            } else {
                break;
            }
        }
        ctx.result()
    }
    fn result(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().map(Ok).unwrap_or(Ok(Value::Unit))
    }
    fn push_frame(&mut self, class: Rc<Class>, ivars: IVars) {
        let frame = StackFrame {
            class,
            offset: self.stack.len(),
            ivars,
        };
        self.frames.push(frame);
    }
    fn pop_frame(&mut self) {
        let frame = self.frames.pop().unwrap();
        let result = self.stack.pop().unwrap();
        self.stack.truncate(frame.offset);
        self.stack.push(result);
    }
    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }
    // TODO: convert frame-relative address to absolute address
    fn local(&self, index: usize) -> usize {
        index + self.frames.last().unwrap().offset
    }
    pub fn get_local(&mut self, index: usize) {
        let idx = self.local(index);
        let value = self.stack[idx].clone();
        self.push(value);
    }
    pub fn get_ivar(&mut self, index: usize) {
        let frame = self.frames.last().unwrap();
        let value = frame.ivars[index].clone();
        self.push(value);
    }
    pub fn assign(&mut self, index: usize) {
        let idx = self.local(index);
        let top = self.stack.pop().unwrap();
        if idx == self.stack.len() {
            self.push(top);
        } else {
            self.stack[idx] = top;
        }
    }
    pub fn send(&mut self, selector: &str, arity: usize) -> Eval {
        let args = self.stack.split_off(self.stack.len() - arity);
        let target = self.stack.pop().unwrap();
        target.send(self, selector, args)
    }
    pub fn object(&mut self, class: &Rc<Class>, arity: usize) -> Eval {
        let ivars = self.stack.split_off(self.stack.len() - arity);
        let obj = Value::Object(class.clone(), Rc::new(ivars));
        self.push(obj);
        Eval::Ok
    }
    pub fn self_object(&mut self, arity: usize) -> Eval {
        let class = &self.frames.last().unwrap().class.clone();
        self.object(&class, arity)
    }
}
