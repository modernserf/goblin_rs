use std::rc::Rc;

use crate::class::{Body, Object, RcClass};
use crate::module_loader::ModuleLoader;
use crate::value::{Handler, Value};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]

pub enum RuntimeError {
    DoesNotUnderstand(String),
    PrimitiveTypeError { expected: String, received: Value },
    // InvalidArg {
    //     expected: Param,
    //     received: Option<Arg>,
    // },
    AssertionError(String),
    UnknownModule(String),
    ModuleLoadLoop(String),
    IndexOutOfRange,
    Panic(Value),
    WithStackTrace(Box<RuntimeError>, Vec<String>),
}

impl RuntimeError {
    pub fn primitive_type_error<T>(expected: &str, received: &Value) -> Runtime<T> {
        Err(RuntimeError::PrimitiveTypeError {
            expected: expected.to_string(),
            received: received.clone(),
        })
    }
    pub fn panic<T>(val: &Value) -> Runtime<T> {
        Err(RuntimeError::Panic(val.clone()))
    }
    pub fn assertion_error<T>(message: &str) -> Runtime<T> {
        Err(RuntimeError::AssertionError(message.to_string()))
    }
    pub fn with_stack_trace(self, trace: Vec<String>) -> RuntimeError {
        RuntimeError::WithStackTrace(Box::new(self), trace)
    }
}
pub type Runtime<T> = Result<T, RuntimeError>;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    // put a value on the stack
    SelfRef,
    Constant(Value),
    Module(String),
    Local { index: usize },
    IVar { index: usize },
    // consume stack values
    Drop,
    SetLocal { index: usize },
    Send { selector: String, arity: usize },
    SendPrimitive { f: NativeHandlerFn, arity: usize },
    TrySend { selector: String, arity: usize },
    NewObject { class: RcClass, arity: usize },
    NewSelf { arity: usize },
    Spawn,
    // control flow
    Return,
}

impl IR {
    #[cfg(test)]
    pub fn int(value: i64) -> IR {
        IR::Constant(Value::Integer(value))
    }
    pub fn send(selector: &str, arity: usize) -> IR {
        IR::Send {
            selector: selector.to_string(),
            arity,
        }
    }
    pub fn send_primitive(f: NativeHandlerFn, arity: usize) -> IR {
        IR::SendPrimitive { f, arity }
    }
    #[cfg(test)]
    pub fn new_object(class: &RcClass, arity: usize) -> IR {
        IR::NewObject {
            class: class.clone(),
            arity,
        }
    }
}

pub type NativeHandlerFn = fn(Value, Vec<Value>) -> Runtime<Value>;

#[derive(Debug)]
struct Stack {
    stack: Vec<Value>,
}

impl Stack {
    fn new() -> Self {
        Stack { stack: Vec::new() }
    }
    fn size(&self) -> usize {
        self.stack.len()
    }
    fn get(&self, index: usize) -> Value {
        self.stack[index].clone()
    }
    fn set(&mut self, index: usize, value: Value) {
        self.stack[index] = value;
    }
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
    fn pop_args(&mut self, count: usize) -> Vec<Value> {
        self.stack.split_off(self.stack.len() - count)
    }
    fn truncate(&mut self, next_length: usize) {
        self.stack.truncate(next_length)
    }
}

#[derive(Debug, Clone)]
enum CallFrameInstance {
    Root,
    Handler(Value),
    Do,
}

#[derive(Debug)]
struct CallFrame {
    selector: String,
    instance: CallFrameInstance,
    body: Body,
    instruction_pointer: usize,
    stack_offset: usize,
}

impl CallFrame {
    fn root(body: Body) -> Self {
        CallFrame {
            selector: "<root>".to_string(),
            instance: CallFrameInstance::Root,
            body,
            instruction_pointer: 0,
            stack_offset: 0,
        }
    }
}

#[derive(Debug)]
struct CallStack {
    stack: Vec<CallFrame>,
    is_unwinding: bool,
}

#[derive(Debug)]
enum NextResult {
    IR(IR),
    Return { offset: usize },
    End,
}

impl CallStack {
    fn new(code: Body) -> Self {
        Self {
            stack: vec![CallFrame::root(code)],
            is_unwinding: false,
        }
    }
    fn top(&self) -> &CallFrame {
        self.stack.last().unwrap()
    }
    fn instance(&self) -> Value {
        for frame in self.stack.iter().rev() {
            match &frame.instance {
                CallFrameInstance::Do => {
                    continue;
                }
                CallFrameInstance::Handler(value) => return value.clone(),
                CallFrameInstance::Root => unreachable!(),
            };
        }
        unreachable!()
    }

    fn call(&mut self, selector: String, offset: usize, handler: Handler) {
        self.stack.push(CallFrame {
            selector,
            stack_offset: offset,
            instruction_pointer: 0,
            body: handler.body(),
            instance: if handler.is_do_block() {
                CallFrameInstance::Do
            } else {
                CallFrameInstance::Handler(handler.instance())
            },
        })
    }

    fn next(&mut self) -> NextResult {
        if self.is_unwinding {
            todo!("drop frames until you get to a handler frame, then set is_unwinding to false")
        }

        let top = self.stack.last_mut().unwrap();
        if top.instruction_pointer < top.body.len() {
            let ir = top.body[top.instruction_pointer].clone();
            top.instruction_pointer += 1;
            return NextResult::IR(ir);
        }
        let result = match top.instance {
            CallFrameInstance::Root => NextResult::End,
            _ => NextResult::Return {
                offset: top.stack_offset,
            },
        };
        self.stack.pop();
        return result;
    }

    fn do_return(&mut self) {
        self.is_unwinding = true;
    }
    fn offset(&self) -> usize {
        self.top().stack_offset
    }
    fn get_ivar(&self, index: usize) -> Value {
        let instance = self.instance();
        instance.ivar(index)
    }
    fn stack_trace(&self) -> Vec<String> {
        self.stack
            .iter()
            .map(|frame| frame.selector.to_string())
            .collect()
    }
}

// #[derive(Debug, Clone, PartialEq)]
// pub enum Arg {
//     Var { index: usize },
//     Do,
// }

// #[derive(Debug)]
// struct Args {
//     args: HashMap<usize, Arg>,
// }

// impl Args {
//     fn new() -> Self {
//         Self {
//             args: HashMap::new(),
//         }
//     }
//     fn var_arg(&mut self, current: usize, parent_index: usize) {
//         self.args.insert(
//             current,
//             Arg::Var {
//                 index: parent_index,
//             },
//         );
//     }
//     fn do_arg(&mut self, current: usize) {
//         self.args.insert(current, Arg::Do);
//     }
//     fn check(&self, index: usize, expected: Param) -> Runtime<()> {
//         let received = self.args.get(&index).cloned();
//         match (&expected, &received) {
//             (Param::Value, None) => {}
//             (Param::Var, Some(Arg::Var { .. })) => {}
//             (Param::Do, None) => {}
//             (Param::Do, Some(Arg::Do)) => {}
//             _ => return Err(RuntimeError::InvalidArg { expected, received }),
//         }
//         Ok(())
//     }
// }

#[allow(unused)]
#[derive(Debug)]
struct Interpreter<'a> {
    stack: Stack,
    call_stack: CallStack,
    // args: Args,
    modules: &'a mut ModuleLoader,
}

pub fn eval_module<'a>(code: Vec<IR>, modules: &'a mut ModuleLoader) -> Runtime<Value> {
    let mut interpreter = Interpreter::new(Rc::new(code), modules);
    interpreter.run()
}

impl<'a> Interpreter<'a> {
    fn new(code: Body, modules: &'a mut ModuleLoader) -> Self {
        Interpreter {
            stack: Stack::new(),
            call_stack: CallStack::new(code),
            // args: Args::new(),
            modules,
        }
    }
    fn run(&mut self) -> Runtime<Value> {
        loop {
            let next = self.call_stack.next();
            match next {
                NextResult::IR(ir) => {
                    self.eval(ir)
                        .map_err(|err| err.with_stack_trace(self.call_stack.stack_trace()))?;
                }
                NextResult::Return { offset } => {
                    let return_value = self.stack.pop();
                    self.stack.truncate(offset);
                    self.stack.push(return_value);
                }
                NextResult::End => {
                    return Ok(self.stack.pop());
                }
            }
        }
    }
    // fn check_args(&self, handler: &Handler) -> Runtime<()> {
    //     let params = handler.params();
    //     let stack_offset = self.stack.size() - params.len();
    //     for (i, param) in params.iter().enumerate() {
    //         self.args.check(i + stack_offset, param.clone())?;
    //     }
    //     Ok(())
    // }
    fn eval(&mut self, ir: IR) -> Runtime<()> {
        match ir {
            // put a value on the stack
            IR::SelfRef => {
                let instance = self.call_stack.instance();
                self.stack.push(instance);
            }
            IR::Constant(value) => {
                self.stack.push(value);
            }
            IR::Module(name) => {
                let result = self.modules.load(&name)?;
                self.stack.push(result);
            }
            IR::Local { index } => {
                let offset = self.call_stack.offset();
                let value = self.stack.get(index + offset);
                self.stack.push(value);
            }
            IR::IVar { index } => {
                let value = self.call_stack.get_ivar(index);
                self.stack.push(value);
            }
            // consume stack values
            IR::Drop => {
                self.stack.pop();
            }
            IR::SetLocal { index } => {
                let value = self.stack.pop();
                let offset = self.call_stack.offset();
                self.stack.set(index + offset, value);
            }
            IR::Send { selector, arity } => {
                let target = self.stack.pop();
                let next_offset = self.stack.size() - arity;
                let handler = target.get_handler(&selector)?;
                // self.check_args(&handler)?;
                self.call_stack.call(selector, next_offset, handler);
            }
            IR::TrySend { selector, arity } => {
                let target = self.stack.pop();
                let or_else = self.stack.pop();
                let next_offset = self.stack.size() - arity;
                if let Ok(handler) = target.get_handler(&selector) {
                    // self.check_args(&handler)?;
                    self.call_stack.call(selector, next_offset, handler);
                } else {
                    self.stack.pop_args(arity);
                    let handler = or_else.get_handler("")?;
                    let next_offset = self.stack.size();
                    self.call_stack.call("".to_string(), next_offset, handler);
                }
            }
            IR::SendPrimitive { f, arity } => {
                let target = self.stack.pop();
                let args = self.stack.pop_args(arity);
                let result = f(target, args)?;
                self.stack.push(result);
            }
            IR::NewObject { class, arity } => {
                let ivars = self.stack.pop_args(arity);
                self.stack
                    .push(Value::Object(Object::new(class, ivars).rc()));
            }
            IR::NewSelf { arity } => {
                let instance = self.call_stack.instance();
                let ivars = self.stack.pop_args(arity);
                self.stack.push(instance.new_instance(ivars));
            }
            IR::Spawn => {
                let target = self.stack.pop();
                let result = eval_module(
                    vec![
                        IR::Constant(target),
                        IR::Send {
                            selector: "".to_string(),
                            arity: 0,
                        },
                    ],
                    self.modules,
                );

                // TODO: return Result<value, error>
                let is_ok = Value::Bool(result.is_ok());
                self.stack.push(is_ok);
            }

            // control flow
            IR::Return => {
                self.call_stack.do_return();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::class::{Class, Param};

    use super::*;

    fn assert_ok(code: Vec<IR>, value: Value) {
        let mut modules = ModuleLoader::new();
        let result = eval_module(code, &mut modules);
        assert_eq!(result, Ok(value));
    }

    fn assert_err(code: Vec<IR>, err: RuntimeError) {
        let mut modules = ModuleLoader::new();
        let result = eval_module(code, &mut modules);
        assert_eq!(result, Err(err));
    }

    #[test]
    fn addition() {
        assert_ok(
            vec![IR::int(1), IR::int(2), IR::send("+:", 1)],
            Value::Integer(3),
        )
    }

    #[test]
    fn does_not_understand() {
        assert_err(
            vec![IR::int(1), IR::send("foo", 0)],
            RuntimeError::DoesNotUnderstand("foo".to_string())
                .with_stack_trace(vec!["<root>".to_string()]),
        )
    }

    #[test]
    fn locals() {
        assert_ok(
            vec![
                IR::int(1),
                IR::Local { index: 0 },
                IR::Local { index: 0 },
                IR::send("+:", 1),
            ],
            Value::Integer(2),
        );
    }

    #[test]
    fn objects() {
        let class = {
            let mut class = Class::new();
            class.add_handler("foo", vec![], vec![IR::int(1)]);
            class.add_handler("bar", vec![], vec![IR::IVar { index: 0 }]);
            class.add_handler(
                "baz:",
                vec![Param::Value],
                vec![
                    IR::IVar { index: 0 },
                    IR::Local { index: 0 },
                    IR::send("+:", 1),
                ],
            );

            class.rc()
        };

        assert_ok(
            vec![IR::int(2), IR::new_object(&class, 1), IR::send("foo", 0)],
            Value::Integer(1),
        );
        assert_ok(
            vec![IR::int(2), IR::new_object(&class, 1), IR::send("bar", 0)],
            Value::Integer(2),
        );
        assert_ok(
            vec![
                IR::int(3),
                IR::int(2),
                IR::new_object(&class, 1),
                IR::send("baz:", 1),
            ],
            Value::Integer(5),
        );
    }

    #[test]
    fn stack_trace() {
        let class = {
            let mut class = Class::new();
            class.add_handler(
                "baz:",
                vec![Param::Value],
                vec![IR::int(1), IR::Local { index: 0 }, IR::send("+:", 1)],
            );

            class.rc()
        };
        assert_err(
            vec![
                IR::Constant(Value::string("hello")),
                IR::new_object(&class, 0),
                IR::send("baz:", 1),
            ],
            RuntimeError::DoesNotUnderstand("+:".to_string())
                .with_stack_trace(vec!["<root>".to_string(), "baz:".to_string()]),
        )
    }
}
