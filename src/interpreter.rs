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
pub enum SendEffect {
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
struct CodeStack {
    code: Vec<Code>,
}

impl CodeStack {
    fn new(program: Body) -> Self {
        Self {
            code: vec![Code::new(program)],
        }
    }
    fn has_code(&self) -> bool {
        !self.code.is_empty()
    }
    fn peek(&self) -> Option<&IR> {
        self.code.last().and_then(|code| code.peek())
    }
    fn next(&mut self) {
        self.code.last_mut().unwrap().next();
    }
    fn push(&mut self, body: Body) {
        self.code.push(Code::new(body));
    }
    fn pop(&mut self) {
        self.code.pop().unwrap();
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
        println!("push @{}", offset);
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
    fn pop(&mut self) -> StackFrame {
        self.frames.pop().unwrap()
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
        let mut code = CodeStack::new(program);
        while code.has_code() {
            while let Some(ir) = code.peek() {
                let result = self.eval(ir);
                code.next();
                if let Some(effect) = result {
                    self.do_effect(&mut code, effect)?;
                }
            }
            let frame = self.frames.pop();
            self.values.return_value(frame.offset());
            code.pop();
        }
        Ok(self.values.pop())
    }
    fn do_effect(&mut self, code: &mut CodeStack, effect: SendEffect) -> Result<(), RuntimeError> {
        match effect {
            SendEffect::Value(value) => {
                self.values.push(value);
            }
            SendEffect::Error(err) => {
                return Err(err);
            }
            SendEffect::Call {
                selector,
                object,
                args,
                body,
                ..
            } => {
                self.frames.push(&mut self.values, object, args);
                code.push(body);
            }
            SendEffect::CallDoBlock {
                parent_object,
                parent_offset,
                args,
                body,
                ..
            } => {
                unimplemented!();
            }
        }
        Ok(())
    }

    fn eval(&mut self, ir: &IR) -> Option<SendEffect> {
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
                let offset = self.frames.offset();
                self.values.assign(index + offset);
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
                let result = target.send(selector, args);
                return Some(result);
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
            IR::Debug(msg) => {
                println!(
                    "{} ({}) {:?}",
                    msg,
                    self.values.values.len(),
                    self.values.values
                );
            }
        };
        return None;
    }
}

#[cfg(test)]
mod test {
    use crate::class::{Class, Handler, Param};

    fn assert_ok(code: Vec<IR>, expected: Value) {
        let result = program(code);
        assert_eq!(result, Ok(expected));
    }

    use super::*;

    #[test]
    fn closure() {
        // let x := 1
        // let y := 2
        // let target := [
        //     on {foo: x}
        //         let y := 3
        //         x + y
        // ]
        // let res := target{foo: 10}
        let code = vec![
            // let x := 1
            IR::Constant(Value::Integer(1)),
            IR::Assign(0),
            // let y := 2
            IR::Constant(Value::Integer(2)),
            IR::Assign(1),
            // let target = [ ... ]
            IR::Object(
                {
                    let mut class = Class::new();
                    // on {foo: x}
                    class.add(
                        "foo:".to_string(),
                        Handler::on(
                            vec![Param::Value],
                            vec![
                                // let y := 3
                                IR::Constant(Value::Integer(3)),
                                IR::Assign(1),
                                // x + y
                                IR::Local(0),
                                IR::Local(1),
                                IR::Send("+:".to_string(), 1),
                            ],
                        ),
                    );
                    class.rc()
                },
                0,
            ),
            IR::Assign(2),
            // let res := target{foo: 10}
            IR::Local(2),
            IR::Constant(Value::Integer(10)),
            IR::Send("foo:".to_string(), 1),
        ];
        assert_ok(code, Value::Integer(13));
    }
}
