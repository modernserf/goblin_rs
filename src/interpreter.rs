use std::rc::Rc;

use crate::{
    class::{Body, Object},
    ir::IR,
    value::Value,
};

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeError {
    DoesNotUnderstand(String),
    PrimitiveTypeError { expected: String, received: Value },
    InvalidArg { expected: String, received: Value },
    AssertionError(String),
}

#[derive(Debug, Clone)]
pub enum Eval {
    Ok,
    Value(Value),
    Error(RuntimeError),
    Call {
        selector: String,
        object: Rc<Object>,
        args: Vec<Value>,
        body: Body,
    },
    CallDoBlock {
        parent_object: Rc<Object>,
        parent_offset: usize,
        args: Vec<Value>,
        body: Body,
    },
}

#[derive(Debug)]
struct Values {
    values: Vec<Value>,
}

impl Values {
    fn new() -> Self {
        Self { values: Vec::new() }
    }
    fn drop(&mut self) {
        self.values.pop().unwrap();
    }
    fn allocate(&mut self, size: usize) {
        for _ in 0..size {
            self.values.push(Value::Unit);
        }
    }
    fn push(&mut self, value: Value) {
        self.values.push(value);
    }
    fn push_local(&mut self, index: usize) {
        let value = self.values[index].clone();
        self.values.push(value);
    }
    fn assign(&mut self, index: usize) {
        // if assigning to top of stack, just leave in place
        if index == self.values.len() - 1 {
            return;
        }
        // pop value from stack & put in new place
        let value = self.values.pop().unwrap();
        self.values[index] = value;
    }
    fn result(&self) -> Value {
        self.values.last().cloned().unwrap_or(Value::Unit)
    }
    fn push_args(&mut self, mut args: Vec<Value>) -> usize {
        let offset = self.values.len();
        self.values.append(&mut args);
        offset
    }
    fn pop(&mut self) -> Value {
        self.values.pop().unwrap()
    }
    fn pop_args(&mut self, arity: usize) -> Vec<Value> {
        self.values.split_off(self.values.len() - arity)
    }
    // fn insert_do_args(&mut self, args: Vec<Value>, offset: usize) {
    //     for (i, arg) in args.into_iter().enumerate() {
    //         self.values[i + offset] = arg;
    //     }
    // }
    fn return_value(&mut self, offset: usize) {
        let value = self.pop();
        self.values.truncate(offset);
        self.values.push(value);
    }
}

#[derive(Debug)]
struct Code {
    index: usize,
    body: Body,
}

impl Code {
    fn new(body: Body) -> Self {
        Self { index: 0, body }
    }
    fn peek(&self) -> Option<&IR> {
        self.body.get(self.index)
    }
    fn next(&mut self) {
        self.index += 1;
    }
}

#[derive(Debug)]
enum StackFrame {
    Root,
    Handler { offset: usize, instance: Rc<Object> },
    // DoHandler {
    //     own_offset: usize,
    //     parent_offset: usize,
    //     parent_instance: Rc<Object>,
    // },
}

#[allow(unused)]
impl StackFrame {
    fn offset(&self) -> usize {
        match self {
            Self::Root { .. } => 0,
            Self::Handler { offset, .. } => *offset,
            // Self::DoHandler { parent_offset, .. } => *parent_offset,
        }
    }
    fn instance(&self) -> Rc<Object> {
        match self {
            Self::Root { .. } => unreachable!(),
            Self::Handler { instance, .. } => instance.clone(),
            // Self::DoHandler {
            //     parent_instance, ..
            // } => parent_instance.clone(),
        }
    }
}

#[derive(Debug)]
struct Frames {
    frames: Vec<StackFrame>,
}

impl Frames {
    fn root() -> Self {
        Self {
            frames: vec![StackFrame::Root],
        }
    }
    fn offset(&self) -> usize {
        self.frames.last().unwrap().offset()
    }
    fn instance(&self) -> Rc<Object> {
        self.frames.last().unwrap().instance()
    }
    fn push(&mut self, stack: &mut Values, instance: Rc<Object>, args: Vec<Value>) {
        let offset = stack.push_args(args);
        self.frames.push(StackFrame::Handler { offset, instance })
    }
    // fn push_do(
    //     &mut self,
    //     stack: &mut Values,
    //     own_offset: usize,
    //     parent_offset: usize,
    //     parent_instance: Rc<Object>,
    //     args: Vec<Value>,
    // ) {
    //     stack.insert_do_args(args, own_offset);
    //     self.frames.push(StackFrame::DoHandler {
    //         own_offset,
    //         parent_offset,
    //         parent_instance,
    //     });
    // }
    fn pop(&mut self, stack: &mut Values) -> Option<Value> {
        let last = self.frames.pop().unwrap();
        match last {
            StackFrame::Root { .. } => {
                let result = stack.result();
                return Some(result);
            }
            StackFrame::Handler { offset, .. } => {
                stack.return_value(offset);
                return None;
            } // StackFrame::DoHandler {
              //     own_offset,
              //     parent_offset,
              //     parent_instance,
              // } => {
              //     unimplemented!()
              // }
        };
    }
}

#[derive(Debug)]
struct Interpreter {
    values: Values,
    frames: Frames,
}

pub fn program(program: Vec<IR>) -> Result<Value, RuntimeError> {
    let mut interpreter = Interpreter::new();
    interpreter.run(Rc::new(program))
}

#[allow(unused)]
impl Interpreter {
    fn new() -> Self {
        Self {
            values: Values::new(),
            frames: Frames::root(),
        }
    }
    fn run(&mut self, program: Body) -> Result<Value, RuntimeError> {
        let mut code_stack = vec![Code::new(program)];
        while let Some(mut code) = code_stack.last_mut() {
            while let Some(ir) = code.peek() {
                let eval = self.eval(ir);
                match eval {
                    Eval::Ok => {
                        code.next();
                    }
                    Eval::Value(value) => {
                        self.values.push(value);
                        code.next();
                    }
                    Eval::Error(err) => {
                        return Err(err);
                    }
                    Eval::Call {
                        selector,
                        object,
                        args,
                        body,
                        ..
                    } => {
                        code.next();
                        self.frames.push(&mut self.values, object, args);
                        *code = Code::new(body);
                    }
                    Eval::CallDoBlock {
                        parent_object,
                        parent_offset,
                        args,
                        body,
                        ..
                    } => {
                        code.next();
                        unimplemented!();
                        // self.frames.push_do();
                        // *code = Code::new(body);
                    }
                }
            }
            if let Some(result) = self.frames.pop(&mut self.values) {
                return Ok(result);
            }
            code_stack.pop().unwrap();
        }
        return Ok(Value::Unit);
    }
    fn eval(&mut self, ir: &IR) -> Eval {
        match ir {
            IR::Drop => {
                self.values.drop();
            }
            IR::Allocate(size) => {
                self.values.allocate(*size);
            }
            IR::Constant(value) => {
                self.values.push(value.clone());
            }
            IR::Assign(index) => {
                self.values.assign(*index);
            }
            IR::Local(index) => {
                let offset = self.frames.offset();
                self.values.push_local(index + offset);
            }
            IR::SelfRef => {
                let instance = self.frames.instance();
                self.values.push(Value::Object(instance));
            }
            IR::IVar(index) => {
                let instance = self.frames.instance();
                let ivar = instance.ivar(*index);
                self.values.push(ivar);
            }
            IR::Send(selector, arity) => {
                let args = self.values.pop_args(*arity);
                let target = self.values.pop();
                return target.send(selector, args);
            }
            IR::Object(class, arity) => {
                let ivars = self.values.pop_args(*arity);
                let obj = Value::Object(Rc::new(Object::new(class.clone(), ivars)));
                self.values.push(obj);
            }
            IR::SelfObject(arity) => {
                let class = self.frames.instance().class();
                let ivars = self.values.pop_args(*arity);
                let obj = Value::Object(Rc::new(Object::new(class.clone(), ivars)));
                self.values.push(obj);
            }
            IR::DoBlock(class) => {
                unimplemented!();
            }
        };
        Eval::Ok
    }
}
