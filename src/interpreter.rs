use std::rc::Rc;

use crate::{
    class::{Body, Object, RcClass},
    ir::IR,
    value::Value,
};

#[derive(Debug)]

enum Frame {
    Root,
    Instance(usize, Rc<Object>),
}

impl Frame {
    fn root() -> Self {
        Frame::Root
    }
    fn offset(&self) -> usize {
        match self {
            Self::Root => 0,
            Self::Instance(offset, _) => *offset,
        }
    }
    fn ivar(&self, index: usize) -> Value {
        match self {
            Self::Root => panic!("no ivars"),
            Self::Instance(_, obj) => obj.ivar(index),
        }
    }
    fn class(&self) -> RcClass {
        match self {
            Self::Root => panic!("no class"),
            Self::Instance(_, obj) => obj.class(),
        }
    }
    fn get_self(&self) -> Value {
        match self {
            Self::Root => panic!("no self"),
            Self::Instance(_, obj) => Value::Object(obj.clone()),
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
        self.0.push(Frame::Instance(offset, object))
    }
    fn pop(&mut self) -> usize {
        let popped = self.0.pop().unwrap();
        popped.offset()
    }
    fn local(&self, index: usize) -> usize {
        self._last().offset() + index
    }
    fn ivar(&self, index: usize) -> Value {
        self._last().ivar(index)
    }
    fn class(&self) -> RcClass {
        self._last().class()
    }
    fn get_self(&self) -> Value {
        self._last().get_self()
    }
}

#[derive(Debug)]
struct Values(Vec<Value>);

impl Values {
    fn new() -> Self {
        Self(Vec::new())
    }
    fn pop_args(&mut self, arity: usize) -> Vec<Value> {
        self.0.split_off(self.0.len() - arity)
    }
    fn push_frame(&mut self, frames: &mut Frames, object: Rc<Object>, mut args: Vec<Value>) {
        let offset = self.0.len();
        frames.push(object, offset);
        self.0.append(&mut args);
    }
    fn pop_frame(&mut self, frames: &mut Frames) {
        let offset = frames.pop();
        let result = self.0.pop().unwrap();
        self.0.truncate(offset);
        self.0.push(result);
    }
    fn push_self(&mut self, frames: &Frames) {
        let value = frames.get_self();
        self.0.push(value);
    }
    fn push(&mut self, value: Value) {
        self.0.push(value);
    }
    fn pop(&mut self) -> Value {
        self.0.pop().unwrap()
    }
    fn local(&mut self, frames: &Frames, index: usize) {
        let index = frames.local(index);
        let val = self.0[index].clone();
        self.0.push(val);
    }
    fn assign(&mut self, frames: &Frames, index: usize) {
        let index = frames.local(index);
        let top = self.0.pop().unwrap();
        if index == self.0.len() {
            self.0.push(top);
        } else {
            self.0[index] = top;
        }
    }
    fn result(&self) -> Value {
        self.0.last().cloned().unwrap_or(Value::Unit)
    }
}

#[derive(Debug)]
struct Code(Vec<(usize, Rc<Vec<IR>>)>);
impl Code {
    fn root(program: Vec<IR>) -> Self {
        Self(vec![(0 as usize, Rc::new(program))])
    }
    fn peek(&self) -> Option<&IR> {
        if let Some((i, body)) = self.0.last() {
            return Some(&body[*i]);
        }
        None
    }
    fn next(&mut self, ctx: &mut Interpreter) {
        let (i, body) = self.0.last_mut().unwrap();
        *i += 1;
        if *i >= body.len() {
            ctx.values.pop_frame(&mut ctx.frames);
            self.0.pop();
        }
    }
    fn push(&mut self, body: Body) {
        self.0.push((0, body))
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
    values: Values,
    frames: Frames,
}

impl Interpreter {
    fn new() -> Self {
        Self {
            values: Values::new(),
            frames: Frames::new(),
        }
    }
    pub fn program(program: Vec<IR>) -> Result<Value, RuntimeError> {
        let mut ctx = Self::new();
        let mut code = Code::root(program);
        while let Some(stmt) = code.peek() {
            match stmt.eval(&mut ctx) {
                Eval::Ok => {
                    code.next(&mut ctx);
                    continue;
                }
                // TODO: add a stack trace here
                Eval::Error(err) => return Err(err),
                Eval::Call { object, args, body } => {
                    code.next(&mut ctx);
                    ctx.values.push_frame(&mut ctx.frames, object, args);
                    code.push(body.clone());
                    continue;
                }
            }
        }
        Ok(ctx.values.result())
    }
    pub fn push(&mut self, value: Value) {
        self.values.push(value)
    }
    pub fn get_local(&mut self, index: usize) {
        self.values.local(&self.frames, index);
    }
    pub fn get_ivar(&mut self, index: usize) {
        let value = self.frames.ivar(index);
        self.values.push(value);
    }
    pub fn assign(&mut self, index: usize) {
        self.values.assign(&self.frames, index);
    }
    pub fn send(&mut self, selector: &str, arity: usize) -> Eval {
        let args = self.values.pop_args(arity);
        let target = self.values.pop();
        target.send(self, selector, args)
    }
    pub fn object(&mut self, class: &RcClass, arity: usize) {
        let ivars = self.values.pop_args(arity);
        let obj = Value::Object(Rc::new(Object::new(class.clone(), ivars)));
        self.values.push(obj);
    }
    pub fn self_object(&mut self, arity: usize) {
        let class = self.frames.class();
        self.object(&class, arity);
    }
    pub fn push_self(&mut self) {
        self.values.push_self(&self.frames);
    }
}
