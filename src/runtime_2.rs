use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    DoesNotUnderstand(Selector),
}
pub type Runtime<T> = Result<T, RuntimeError>;

pub type Address = usize;
pub type Selector = String;
pub type Index = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Unit,                // (-- value)
    Integer(i64),        // (-- value)
    Local(Address),      // ( -- *address)
    Var(Address),        // ( -- address)
    IVal(Index),         // ( -- instance[index])
    Object(Rc<Class>),   // (...instance -- object)
    DoObject(Rc<Class>), // (...instance -- object)
    Deref,               // (address -- *address)
    SetVar,              // (value address -- )
    Send(Selector),      // (...args target -- result)
    Drop,                // (value --)
    Return,
}

type Body = Rc<Vec<IR>>;
type Instance = Rc<Vec<Value>>;
type ParentFrameIndex = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Integer(i64),
    Object(Rc<Class>, Instance),
    DoObject(Rc<Class>, Instance, ParentFrameIndex),
    Pointer(Address),
}

impl Value {
    fn as_int(self) -> i64 {
        match self {
            Value::Integer(val) => val,
            _ => panic!("cannot cast to int"),
        }
    }
    fn primitive_add(self, arg: Value) -> Value {
        Value::Integer(self.as_int() + arg.as_int())
    }
    fn send(self, selector: &str, stack: &mut Stack, call_stack: &mut CallStack) -> Runtime<()> {
        match self {
            Value::Unit => Err(RuntimeError::DoesNotUnderstand(selector.to_string())),
            Value::Integer(_) => match selector {
                "+:" => {
                    let arg = stack.pop();
                    let result = self.primitive_add(arg);
                    stack.push(result);
                    Ok(())
                }
                _ => Err(RuntimeError::DoesNotUnderstand(selector.to_string())),
            },
            Value::Object(class, ivals) => {
                let handler = class.get(selector)?;
                let local_offset = stack.size();
                call_stack.call(handler, local_offset, ivals);
                Ok(())
            }
            Value::DoObject(class, ivals, return_from_index) => {
                let handler = class.get(selector)?;
                let local_offset = stack.size();
                call_stack.call_do(handler, local_offset, ivals, return_from_index);
                Ok(())
            }
            Value::Pointer(_) => panic!("must deref pointer before sending message"),
        }
    }
    fn deref(self, stack: &Stack) -> Value {
        match self {
            Value::Pointer(address) => stack.get(address),
            _ => panic!("deref a non-pointer"),
        }
    }
    fn set(self, value: Value, stack: &mut Stack) {
        match self {
            Value::Pointer(address) => stack.set(address, value),
            _ => panic!("assign a non-pointer"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    handlers: HashMap<Selector, Handler>,
    ivals: usize,
}

impl Class {
    pub fn new() -> Self {
        Class {
            handlers: HashMap::new(),
            ivals: 0,
        }
    }
    #[cfg(test)]
    pub fn add(&mut self, selector: &str, params: Vec<Param>, body: Vec<IR>) {
        self.add_handler(selector.to_string(), params, body)
    }
    pub fn add_handler(&mut self, selector: String, params: Vec<Param>, body: Vec<IR>) {
        self.handlers.insert(
            selector,
            Handler {
                body: Rc::new(body),
                params,
            },
        );
    }
    fn get(&self, selector: &str) -> Runtime<&Handler> {
        match self.handlers.get(selector) {
            Some(handler) => Ok(handler),
            None => Err(RuntimeError::DoesNotUnderstand(selector.to_string())),
        }
    }
    fn get_ivals(&self) -> usize {
        self.ivals
    }
    pub fn set_ivals(&mut self, count: usize) {
        self.ivals = count
    }
    pub fn rc(self) -> Rc<Class> {
        Rc::new(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Handler {
    params: Vec<Param>,
    body: Body,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Param {
    Value,
    Var,
    Do,
}

struct Stack {
    stack: Vec<Value>,
}

impl Stack {
    fn new() -> Self {
        Stack { stack: Vec::new() }
    }
    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
    fn get(&self, index: Address) -> Value {
        self.stack[index].clone()
    }
    fn set(&mut self, index: Address, value: Value) {
        self.stack[index] = value
    }
    fn size(&self) -> usize {
        self.stack.len()
    }
    fn truncate(&mut self, offset: usize) {
        self.stack.truncate(offset);
    }
    fn take(&mut self, count: usize) -> Vec<Value> {
        self.stack.split_off(self.stack.len() - count)
    }
}

enum Frame {
    Root {
        body: Body,
        ip: usize,
    },
    Handler {
        body: Body,
        ip: usize,
        local_offset: usize,
        ivals: Instance,
        return_from_index: usize,
    },
}

impl Frame {
    fn root(code: Vec<IR>) -> Self {
        Frame::Root {
            body: Rc::new(code),
            ip: 0,
        }
    }
    fn local_offset(&self) -> usize {
        match self {
            Frame::Root { .. } => 0,
            Frame::Handler { local_offset, .. } => *local_offset,
        }
    }
    fn ival(&self, index: usize) -> Value {
        match self {
            Frame::Root { .. } => panic!("root has no ivals"),
            Frame::Handler { ivals, .. } => ivals[index].clone(),
        }
    }
    fn next(&mut self) -> NextResult {
        match self {
            Frame::Root { body, ip } => {
                if *ip >= body.len() {
                    return NextResult::Done;
                }
                let res = NextResult::IR(body[*ip].clone());
                *ip += 1;
                return res;
            }
            Frame::Handler {
                body,
                ip,
                local_offset,
                ..
            } => {
                if *ip >= body.len() {
                    return NextResult::Return(*local_offset);
                }
                let res = NextResult::IR(body[*ip].clone());
                *ip += 1;
                return res;
            }
        }
    }
    fn return_from_index(&self) -> usize {
        match self {
            Frame::Root { .. } => 0,
            Frame::Handler {
                return_from_index, ..
            } => *return_from_index,
        }
    }
}

enum NextState {
    Init,
    Return,
}

struct CallStack {
    frames: Vec<Frame>,
    next_state: NextState,
}

impl CallStack {
    fn root(code: Vec<IR>) -> Self {
        CallStack {
            frames: vec![Frame::root(code)],
            next_state: NextState::Init,
        }
    }
    fn next(&mut self) -> NextResult {
        if let NextState::Return = self.next_state {
            self.next_state = NextState::Init;
            let return_from_index = self.return_from_index();
            if return_from_index == 0 {
                return NextResult::Done;
            }
            self.frames.truncate(return_from_index + 1);
            let last_frame = self.frames.pop().unwrap();
            let offset = last_frame.local_offset();
            return NextResult::Return(offset);
        }

        let frame = self.top_mut();
        let res = frame.next();
        if let NextResult::Return(_) = res {
            self.frames.pop();
        }
        res
    }
    fn local_offset(&self) -> usize {
        self.top().local_offset()
    }
    fn top(&self) -> &Frame {
        self.frames.last().unwrap()
    }
    fn top_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }
    fn call(&mut self, handler: &Handler, local_offset: usize, ivals: Instance) {
        let return_from_index = self.frames.len();
        self.frames.push(Frame::Handler {
            body: handler.body.clone(),
            ip: 0,
            local_offset: local_offset - handler.params.len(),
            ivals,
            return_from_index,
        })
    }
    fn call_do(
        &mut self,
        handler: &Handler,
        local_offset: usize,
        ivals: Instance,
        return_from_index: usize,
    ) {
        self.frames.push(Frame::Handler {
            body: handler.body.clone(),
            ip: 0,
            local_offset: local_offset - handler.params.len(),
            ivals,
            return_from_index,
        })
    }
    fn ival(&self, index: usize) -> Value {
        self.top().ival(index)
    }
    fn return_from_index(&self) -> usize {
        self.top().return_from_index()
    }
    fn do_return(&mut self) {
        self.next_state = NextState::Return
    }
}

enum NextResult {
    IR(IR),
    Return(usize),
    Done,
}

pub struct Interpreter {
    stack: Stack,
    call_stack: CallStack,
}

impl Interpreter {
    pub fn program(code: Vec<IR>) -> Runtime<Value> {
        let mut interpreter = Interpreter {
            stack: Stack::new(),
            call_stack: CallStack::root(code),
        };
        interpreter.run()
    }
    fn run(&mut self) -> Runtime<Value> {
        loop {
            match self.call_stack.next() {
                NextResult::IR(ir) => self.eval(ir)?,
                NextResult::Return(offset) => {
                    let value = self.stack.pop();
                    self.stack.truncate(offset);
                    self.stack.push(value);
                }
                NextResult::Done => return Ok(self.stack.pop()),
            };
        }
    }
    fn eval(&mut self, ir: IR) -> Runtime<()> {
        match ir {
            IR::Unit => self.stack.push(Value::Unit),
            IR::Integer(value) => self.stack.push(Value::Integer(value)),
            IR::Local(address) => {
                let local_offset = self.call_stack.local_offset();
                let value = self.stack.get(address + local_offset);
                self.stack.push(value);
            }
            IR::IVal(index) => {
                let value = self.call_stack.ival(index);
                self.stack.push(value);
            }
            IR::Var(address) => {
                let absolute_address = address + self.call_stack.local_offset();
                self.stack.push(Value::Pointer(absolute_address));
            }
            IR::Object(class) => {
                let ivals = Rc::new(self.stack.take(class.get_ivals()));
                let value = Value::Object(class, ivals);
                self.stack.push(value);
            }
            IR::DoObject(class) => {
                let ivals = Rc::new(self.stack.take(class.get_ivals()));
                let return_from_index = self.call_stack.return_from_index();
                let value = Value::DoObject(class, ivals, return_from_index);
                self.stack.push(value);
            }
            IR::Deref => {
                let pointer = self.stack.pop();
                let value = pointer.deref(&self.stack);
                self.stack.push(value);
            }
            IR::SetVar => {
                let pointer = self.stack.pop();
                let value = self.stack.pop();
                pointer.set(value, &mut self.stack);
            }
            IR::Send(selector) => {
                let target = self.stack.pop();
                target.send(&selector, &mut self.stack, &mut self.call_stack)?;
            }
            IR::Return => self.call_stack.do_return(),
            IR::Drop => {
                self.stack.pop();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_ok(code: Vec<IR>, expected: Value) {
        assert_eq!(Interpreter::program(code), Ok(expected));
    }

    fn assert_err(code: Vec<IR>, expected: RuntimeError) {
        assert_eq!(Interpreter::program(code), Err(expected));
    }

    fn add() -> IR {
        IR::Send("+:".to_string())
    }

    #[test]
    fn addition() {
        assert_ok(
            vec![IR::Integer(1), IR::Integer(2), add()],
            Value::Integer(3),
        )
    }

    #[test]
    fn does_not_understand() {
        assert_err(
            vec![IR::Integer(1), IR::Send("foobar".to_string())],
            RuntimeError::DoesNotUnderstand("foobar".to_string()),
        )
    }

    #[test]
    fn locals() {
        assert_ok(
            vec![
                IR::Integer(123), // 0
                IR::Integer(456), // 1
                IR::Integer(789), // 2
                IR::Local(1),
                IR::Integer(10),
                add(),
            ],
            Value::Integer(466),
        )
    }

    #[test]
    fn variables() {
        assert_ok(
            vec![
                IR::Integer(123), // 0
                IR::Integer(1),
                IR::Local(0),
                add(),
                IR::Var(0),
                IR::SetVar,
                IR::Local(0),
            ],
            Value::Integer(124),
        )
    }

    fn empty() -> Instance {
        Rc::new(vec![])
    }

    #[test]
    fn objects() {
        let empty_class = Class::new().rc();
        assert_ok(
            vec![
                IR::Object(empty_class.clone()), // 0
                IR::Integer(1),                  // 1
                IR::Local(0),
            ],
            Value::Object(empty_class, empty()),
        )
    }

    #[test]
    fn object_does_not_understand() {
        let empty_class = Class::new().rc();
        assert_err(
            vec![
                IR::Object(empty_class.clone()),
                IR::Send("foobar".to_string()),
            ],
            RuntimeError::DoesNotUnderstand("foobar".to_string()),
        )
    }

    #[test]
    fn simple_handlers() {
        let record = {
            let mut class = Class::new();
            class.add("x", vec![], vec![IR::Integer(1)]);
            class.add("y", vec![], vec![IR::Integer(2)]);
            class.rc()
        };
        assert_ok(
            vec![
                IR::Object(record.clone()), // 0
                IR::Local(0),
                IR::Send("x".to_string()),
                IR::Local(0),
                IR::Send("y".to_string()),
                add(),
            ],
            Value::Integer(3),
        )
    }

    #[test]
    fn handler_locals() {
        let foo = {
            let mut class = Class::new();
            class.add(
                "foo",
                vec![],
                vec![
                    IR::Integer(123), // 0,
                    IR::Integer(100), // 1,
                    IR::Local(0),
                    IR::Local(1),
                    add(),
                ],
            );
            class.rc()
        };
        assert_ok(
            vec![
                IR::Integer(69),         // 0
                IR::Integer(420),        // 1
                IR::Object(foo.clone()), // 2
                IR::Local(2),
                IR::Send("foo".to_string()),
            ],
            Value::Integer(223),
        )
    }

    #[test]
    fn handler_local_vars() {
        let foo = {
            let mut class = Class::new();
            class.add(
                "foo",
                vec![],
                vec![
                    IR::Integer(123), // 0,
                    IR::Integer(100), // 1,
                    IR::Local(0),
                    IR::Local(1),
                    add(),
                    IR::Var(0),
                    IR::SetVar, // $0 = $1 + $0
                    IR::Local(0),
                    IR::Local(1),
                    add(),
                ],
            );
            class.rc()
        };
        assert_ok(
            vec![
                IR::Integer(69),  // 0
                IR::Integer(420), // 1
                IR::Object(foo),  // 2
                IR::Local(2),
                IR::Send("foo".to_string()),
            ],
            Value::Integer(323),
        )
    }

    #[test]
    fn handler_args() {
        let double_then_add_10 = {
            let mut class = Class::new();
            class.add(
                "foo:",
                vec![Param::Value],
                vec![
                    IR::Local(0),
                    IR::Local(0),
                    add(), // let $1 = $0 + $0
                    IR::Local(1),
                    IR::Integer(10),
                    add(), // return $1 + 10
                ],
            );

            class.rc()
        };
        assert_ok(
            vec![
                IR::Integer(50),                        // $0
                IR::Object(double_then_add_10.clone()), // $1
                IR::Local(0),
                IR::Local(1),
                IR::Send("foo:".to_string()), // $1{foo: $0}
            ],
            Value::Integer(110),
        )
    }

    #[test]
    fn arg_cleanup() {
        let double_then_add_10 = {
            let mut class = Class::new();
            class.add(
                "foo:",
                vec![Param::Value],
                vec![
                    IR::Local(0),
                    IR::Local(0),
                    add(), // let $1 = $0 + $0
                    IR::Local(1),
                    IR::Integer(10),
                    add(), // return $1 + 10
                ],
            );

            class.rc()
        };

        assert_ok(
            vec![
                IR::Integer(50),                // $0
                IR::Object(double_then_add_10), // $1
                IR::Local(0),
                IR::Local(1),
                IR::Send("foo:".to_string()), // $2 = $1{foo: $0}
                IR::Local(2),
            ],
            Value::Integer(110),
        );
    }

    #[test]
    fn var_args() {
        let add_10_to_var_arg = {
            let mut class = Class::new();
            class.add(
                "foo:",
                vec![Param::Var],
                vec![
                    IR::Integer(10),
                    IR::Local(0),
                    IR::Deref,
                    add(),
                    IR::Local(0),
                    IR::SetVar,     // $0 = *$0 + 10
                    IR::Integer(0), // (ignore return value)
                ],
            );

            class.rc()
        };

        assert_ok(
            vec![
                IR::Object(add_10_to_var_arg), // $0
                IR::Integer(100),              // $1
                IR::Var(1),
                IR::Local(0),
                IR::Send("foo:".to_string()), // $0{foo: var $1}
                IR::Local(1),
            ],
            Value::Integer(110),
        )
    }

    #[test]
    fn instance_values() {
        let pair = {
            let mut class = Class::new();
            class.add("x", vec![], vec![IR::IVal(0)]);
            class.add("y", vec![], vec![IR::IVal(1)]);
            class.set_ivals(2);
            class.rc()
        };

        assert_ok(
            vec![
                IR::Integer(1),
                IR::Integer(2),
                IR::Object(pair.clone()), // $0 = [x: 1 y: 2]
                IR::Integer(3),
                IR::Integer(4),
                IR::Object(pair), // $1 = [x: 3 y: 4]
                IR::Local(0),
                IR::Send("x".to_string()),
                IR::Local(1),
                IR::Send("y".to_string()),
                add(), // $0{x} + $1{y}
            ],
            Value::Integer(5),
        )
    }

    #[test]
    fn set_instance_value_var() {
        assert_ok(
            vec![
                IR::Integer(100), // $0
                IR::Var(0),
                IR::Object({
                    let mut class = Class::new();
                    class.set_ivals(1);
                    class.add(
                        "add to var:",
                        vec![Param::Value],
                        vec![
                            IR::IVal(0),
                            IR::Deref,
                            IR::Local(0),
                            add(),
                            IR::IVal(0),
                            IR::SetVar,
                            IR::Integer(0),
                        ],
                    );
                    class.rc()
                }), // $1
                IR::Integer(20),
                IR::Local(1),
                IR::Send("add to var:".to_string()),
                IR::Local(0),
            ],
            Value::Integer(120),
        )
    }

    #[test]
    fn return_from_handler() {
        let obj = {
            let mut class = Class::new();
            class.add(
                "add 10:",
                vec![Param::Value],
                vec![
                    IR::Integer(10),
                    IR::Local(0),
                    add(),
                    IR::Return,
                    // should be unreachable
                    IR::Integer(20),
                ],
            );
            class.rc()
        };
        assert_ok(
            vec![
                IR::Integer(3),
                IR::Object(obj),
                IR::Send("add 10:".to_string()),
            ],
            Value::Integer(13),
        );
    }

    #[test]
    fn return_from_root() {
        assert_ok(
            vec![
                IR::Integer(3),
                IR::Return,
                // should be unreachable
                IR::Integer(4),
            ],
            Value::Integer(3),
        );
    }

    #[test]
    fn return_from_do_object() {
        /*
          [
            on {run}
              let obj := [
                on {match: do f}
                  f{some: 50}
                  456
              ]
              obj{match:
                on {some: value}
                  return value + value
                  123
              }
              789
          ]{run}
        */
        assert_ok(
            vec![
                IR::Object({
                    let mut class = Class::new();
                    class.add(
                        "run",
                        vec![],
                        vec![
                            IR::Object({
                                let mut class = Class::new();
                                class.add(
                                    "match:",
                                    vec![Param::Do],
                                    vec![
                                        IR::Integer(50),
                                        IR::Local(0),
                                        IR::Send("some:".to_string()),
                                        // unreachable if do block returns early
                                        IR::Integer(456),
                                    ],
                                );
                                class.rc()
                            }), // $0
                            IR::DoObject({
                                let mut class = Class::new();
                                class.add(
                                    "some:",
                                    vec![Param::Value],
                                    vec![
                                        IR::Local(0),
                                        IR::Local(0),
                                        add(),
                                        IR::Return,
                                        // unreachable
                                        IR::Integer(123),
                                    ],
                                );
                                class.rc()
                            }),
                            IR::Local(0),
                            IR::Send("match:".to_string()),
                            // unreachable if match do arg returns early
                            IR::Integer(789),
                        ],
                    );
                    class.rc()
                }),
                IR::Send("run".to_string()),
            ],
            Value::Integer(100),
        );
    }
}
