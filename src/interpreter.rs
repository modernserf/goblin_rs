use std::{collections::HashMap, rc::Rc};

use crate::{
    class::{Body, Class, Object},
    ir::IR,
    runtime_error::RuntimeError,
    value::Value,
};

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
        own_offset: usize,
        parent_index: usize,
        args: Vec<Value>,
        body: Body,
    },
    Return,
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
    fn push_var_arg(&mut self, index: usize) {
        let value = self.values[index].clone();
        self.values.push(Value::Var(index, Box::new(value)));
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
    fn insert_do_args(&mut self, args: Vec<Value>, offset: usize) {
        for (i, arg) in args.into_iter().enumerate() {
            self.values[i + offset] = arg;
        }
    }
    fn return_value(&mut self, offset: usize) {
        let value = self.pop();
        self.values.truncate(offset);
        self.values.push(value);
    }
    fn update_vars(&mut self, offset: usize, args: Vec<Value>) {
        for (i, arg) in args.into_iter().enumerate() {
            if let Value::Var(absolute_index, _) = arg {
                self.values[absolute_index] = self.values[offset + i].clone();
            }
        }
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
    Handler {
        offset: usize,
        instance: Rc<Object>,
        args: Vec<Value>,
    },
    DoHandler {
        parent_index: usize,
        parent_offset: usize,
        parent_instance: Rc<Object>,
        args: Vec<Value>,
    },
}

thread_local! {
    static NIL_INSTANCE: Rc<Object> = Rc::new(Object::new(Class::new().rc(), vec![]));
}

#[allow(unused)]
impl StackFrame {
    fn offset(&self) -> usize {
        match self {
            Self::Root { .. } => 0,
            Self::Handler { offset, .. } => *offset,
            Self::DoHandler { parent_offset, .. } => *parent_offset,
        }
    }
    fn instance(&self) -> Rc<Object> {
        match self {
            Self::Root { .. } => NIL_INSTANCE.with(|x| x.clone()),
            Self::Handler { instance, .. } => instance.clone(),
            Self::DoHandler {
                parent_instance, ..
            } => parent_instance.clone(),
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
    fn index(&self) -> usize {
        self.frames.len() - 1
    }
    fn push(&mut self, stack: &mut Values, instance: Rc<Object>, args: Vec<Value>) {
        let offset = stack.push_args(args.clone());
        self.frames.push(StackFrame::Handler {
            offset,
            instance,
            args,
        })
    }
    fn push_do(
        &mut self,
        stack: &mut Values,
        own_offset: usize,
        parent_index: usize,
        args: Vec<Value>,
    ) {
        let parent_frame = &self.frames[parent_index];
        let parent_offset = parent_frame.offset();
        let parent_instance = parent_frame.instance();
        stack.insert_do_args(args.clone(), own_offset);
        self.frames.push(StackFrame::DoHandler {
            parent_index,
            parent_offset,
            parent_instance,
            args,
        });
    }
    fn pop(&mut self, stack: &mut Values) {
        match self.frames.pop().unwrap() {
            StackFrame::Root => {
                stack.return_value(0);
            }
            StackFrame::Handler { offset, args, .. } => {
                stack.update_vars(offset, args);
                stack.return_value(offset);
            }
            StackFrame::DoHandler {
                args,
                parent_offset,
                ..
            } => {
                stack.update_vars(parent_offset, args);
                // return value is on top of stack
                // args & locals are in allocated space
                // nothing needs to be moved / cleaned up
            }
        }
    }
    fn return_deep(&mut self, stack: &mut Values) -> usize {
        match self.frames.pop().unwrap() {
            StackFrame::Root => {
                stack.return_value(0);
                1
            }
            StackFrame::Handler { offset, args, .. } => {
                stack.update_vars(offset, args);
                stack.return_value(offset);
                1
            }
            StackFrame::DoHandler {
                parent_offset,
                args,
                parent_index,
                ..
            } => {
                stack.update_vars(parent_offset, args);
                let return_value = stack.pop();
                let mut depth = 1;
                while self.index() >= parent_index {
                    self.pop_without_return(stack);
                    depth += 1;
                }
                stack.push(return_value);
                depth
            }
        }
    }
    fn pop_without_return(&mut self, stack: &mut Values) {
        match self.frames.pop().unwrap() {
            StackFrame::Root => {}
            StackFrame::Handler { offset, args, .. } => {
                stack.update_vars(offset, args);
            }
            StackFrame::DoHandler {
                args,
                parent_offset,
                ..
            } => {
                stack.update_vars(parent_offset, args);
            }
        }
    }
}

#[derive(Debug)]
struct Interpreter {
    values: Values,
    frames: Frames,
    modules: HashMap<String, Value>,
}

pub fn program(program: Vec<IR>, modules: HashMap<String, Value>) -> Result<Value, RuntimeError> {
    let mut interpreter = Interpreter::new(modules);
    interpreter.run(Rc::new(program))
}

#[allow(unused)]
impl Interpreter {
    fn new(modules: HashMap<String, Value>) -> Self {
        Self {
            values: Values::new(),
            frames: Frames::root(),
            modules: modules,
        }
    }
    fn run(&mut self, program: Body) -> Result<Value, RuntimeError> {
        let mut code = CodeStack::new(program);
        while code.has_code() {
            let mut returned = false;
            while let Some(ir) = code.peek() {
                let result = self.eval(ir);
                code.next();
                if let Some(effect) = result {
                    if let SendEffect::Return = effect {
                        returned = true;
                        break;
                    }
                    self.do_effect(&mut code, effect)?;
                }
            }
            if returned {
                let return_depth = self.frames.return_deep(&mut self.values);
                for _ in 0..return_depth {
                    code.pop();
                }
            } else {
                self.frames.pop(&mut self.values);
                code.pop();
            }
        }
        Ok(self.values.pop())
    }
    fn do_effect(&mut self, code: &mut CodeStack, effect: SendEffect) -> Result<(), RuntimeError> {
        match effect {
            SendEffect::Return => {
                unreachable!()
            }
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
                own_offset,
                parent_index,
                args,
                body,
                ..
            } => {
                self.frames
                    .push_do(&mut self.values, own_offset, parent_index, args);
                code.push(body);
            }
        }
        Ok(())
    }

    fn eval(&mut self, ir: &IR) -> Option<SendEffect> {
        match ir {
            IR::Drop => {
                self.values.drop();
            }
            IR::Return => return Some(SendEffect::Return),
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
            IR::VarArg(index) => {
                let offset = self.frames.offset();
                self.values.push_var_arg(index + offset);
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
            IR::SendPrimitive(f, arity) => {
                let args = self.values.pop_args(*arity);
                let target = self.values.pop();
                let result = f(target, args);
                return Some(result);
            }
            IR::TrySend(selector, arity) => {
                let or_else = self.values.pop();
                let args = self.values.pop_args(*arity);
                let target = self.values.pop();
                let result = target.send(selector, args);
                return match &result {
                    // TODO: check that this doesn't "bubble"
                    SendEffect::Error(RuntimeError::DoesNotUnderstand(_)) => {
                        Some(or_else.send("", vec![]))
                    }
                    _ => Some(result),
                };
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
            IR::DoBlock { class, own_offset } => {
                let obj = Value::Do {
                    class: class.clone(),
                    own_offset: *own_offset,
                    parent_index: self.frames.index(),
                };
                self.values.push(obj);
            }
            IR::Debug(msg) => {
                println!(
                    "{} ({}) {:?}",
                    msg,
                    self.values.values.len(),
                    self.values.values
                );
            }
            IR::Module(name) => {
                if let Some(module) = self.modules.get(name) {
                    self.values.push(module.clone());
                } else {
                    return Some(RuntimeError::unknown_module(name));
                }
            }
        };
        return None;
    }
}

#[cfg(test)]
mod test {
    use crate::class::{Class, Param};

    fn assert_ok(code: Vec<IR>, expected: Value) {
        let result = program(code, HashMap::new());
        assert_eq!(result, Ok(expected));
    }

    use super::*;

    #[test]
    fn primitive() {
        let code = vec![
            IR::Constant(Value::Integer(1)),
            IR::Constant(Value::Integer(2)),
            IR::Send("+:".to_string(), 1),
        ];
        assert_ok(code, Value::Integer(3));
    }

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
                    class.add_handler(
                        "foo:",
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
