use std::rc::Rc;

use crate::{
    class::{Body, Object, RcClass},
    ir::IR,
    value::Value,
};

#[derive(Debug)]
pub struct Frame {
    object: Rc<Object>,
    offset: usize,
}

impl Frame {
    fn root() -> Self {
        Frame {
            object: Rc::new(Object::empty()),
            offset: 0,
        }
    }
}

#[derive(Debug)]
struct Frames(Vec<Frame>);

impl Frames {
    fn new() -> Self {
        Self(vec![Frame::root()])
    }
    fn _last(&self) -> &Frame {
        self.0.last().unwrap()
    }
    fn push(&mut self, object: Rc<Object>, offset: usize) {
        self.0.push(Frame { object, offset })
    }
    fn pop(&mut self) -> usize {
        let popped = self.0.pop().unwrap();
        popped.offset
    }
    fn local(&self, index: usize) -> usize {
        self._last().offset + index
    }
    fn ivar(&self, index: usize) -> Value {
        self._last().object.ivar(index)
    }
    fn class(&self) -> RcClass {
        self._last().object.class().clone()
    }
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
        object: Rc<Object>,
        args: Vec<Value>,
        body: Body,
    },
}

#[derive(Debug)]
pub struct Interpreter {
    stack: Vec<Value>,
    frames: Frames,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: Frames::new(),
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
                            object,
                            mut args,
                            body,
                        } => {
                            *i += 1;
                            let body = body.clone();
                            ctx.push_frame(object);
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
    fn push_frame(&mut self, object: Rc<Object>) {
        self.frames.push(object, self.stack.len());
    }
    fn pop_frame(&mut self) {
        let offset = self.frames.pop();
        let result = self.stack.pop().unwrap();
        self.stack.truncate(offset);
        self.push(result);
    }
    pub fn push(&mut self, value: Value) {
        self.stack.push(value)
    }
    fn local(&self, index: usize) -> usize {
        self.frames.local(index)
    }
    pub fn get_local(&mut self, index: usize) {
        let idx = self.local(index);
        let value = self.stack[idx].clone();
        self.push(value);
    }
    pub fn get_ivar(&mut self, index: usize) {
        let value = self.frames.ivar(index);
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
    pub fn object(&mut self, class: &RcClass, arity: usize) -> Eval {
        let ivars = self.stack.split_off(self.stack.len() - arity);
        let obj = Value::Object(Rc::new(Object::new(class.clone(), ivars)));
        self.push(obj);
        Eval::Ok
    }
    pub fn self_object(&mut self, arity: usize) -> Eval {
        let class = self.frames.class();
        self.object(&class, arity)
    }
}
