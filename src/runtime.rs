use std::{collections::HashMap, rc::Rc};

use crate::ir::{Address, Handler, Selector, Value, IR};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeError {
    DoesNotUnderstand(Selector),
    ExpectedVarArg,
    DidNotExpectDoArg,
    ExpectedType(String),
    ModuleLoadLoop(String),
    UnknownModule(String),
    Panic(String),
}
pub type Runtime<T> = Result<T, RuntimeError>;

#[derive(Debug, Clone)]
enum ModuleLoadState {
    Init(Vec<IR>),
    Loading,
    Ready(Value),
}

#[derive(Debug, Clone)]
pub struct ModuleLoader {
    modules: HashMap<String, ModuleLoadState>,
}

impl ModuleLoader {
    pub fn new() -> Self {
        ModuleLoader {
            modules: HashMap::new(),
        }
    }
    pub fn add_init(&mut self, name: &str, ir: Vec<IR>) {
        self.modules
            .insert(name.to_string(), ModuleLoadState::Init(ir));
    }
    pub fn add_ready(&mut self, name: &str, value: Value) {
        self.modules
            .insert(name.to_string(), ModuleLoadState::Ready(value));
    }
    pub fn load(&mut self, name: &str) -> Runtime<Value> {
        match self.modules.get_mut(name) {
            Some(ModuleLoadState::Loading) => Err(RuntimeError::ModuleLoadLoop(name.to_string())),
            Some(ModuleLoadState::Ready(value)) => Ok(value.clone()),
            Some(ModuleLoadState::Init(ir)) => {
                let ir = std::mem::take(ir);
                self.modules
                    .insert(name.to_string(), ModuleLoadState::Loading);

                match Interpreter::program(ir, self) {
                    Ok(value) => {
                        self.add_ready(name, value.clone());
                        Ok(value)
                    }
                    Err(err) => Err(err),
                }
            }
            None => Err(RuntimeError::UnknownModule(name.to_string())),
        }
    }
}

enum Frame {
    Root {
        body: Vec<IR>,
        ip: usize,
    },
    Handler {
        handler: Rc<Handler>,
        ip: usize,
        local_offset: usize,
        self_value: Value,
        target_value: Value,
        return_from_index: usize,
    },
}

impl Frame {
    fn root(code: Vec<IR>) -> Self {
        Frame::Root { body: code, ip: 0 }
    }
    fn local_offset(&self) -> usize {
        match self {
            Frame::Root { .. } => 0,
            Frame::Handler { local_offset, .. } => *local_offset,
        }
    }
    fn self_value(&self) -> Value {
        match self {
            Frame::Root { .. } => Value::Unit,
            Frame::Handler { self_value, .. } => self_value.clone(),
        }
    }
    fn ival(&self, index: usize) -> Value {
        match self {
            Frame::Root { .. } => panic!("root has no ivals"),
            Frame::Handler { target_value, .. } => target_value.ival(index),
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
                res
            }
            Frame::Handler {
                handler,
                ip,
                local_offset,
                ..
            } => {
                if *ip >= handler.body.len() {
                    return NextResult::Return(*local_offset);
                }
                let res = NextResult::IR(handler.body[*ip].clone());
                *ip += 1;
                res
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
    fn do_loop(&mut self) {
        match self {
            Frame::Root { ip, .. } => *ip = 0,
            Frame::Handler { ip, .. } => *ip = 0,
        }
    }
}

enum NextState {
    Init,
    Return,
}

enum NextResult {
    IR(IR),
    Return(usize),
    Done,
}

pub struct Interpreter<'a> {
    stack: Vec<Value>,
    frames: Vec<Frame>,
    next_state: NextState,
    modules: &'a mut ModuleLoader,
}

impl<'a> Interpreter<'a> {
    pub fn program(code: Vec<IR>, modules: &'a mut ModuleLoader) -> Runtime<Value> {
        let mut interpreter = Interpreter {
            stack: Vec::with_capacity(1024),
            frames: {
                let mut frames = Vec::with_capacity(64);
                frames.push(Frame::root(code));
                frames
            },
            next_state: NextState::Init,
            modules,
        };
        interpreter.run()
    }
    fn run(&mut self) -> Runtime<Value> {
        loop {
            match self.next() {
                NextResult::IR(ir) => ir.eval(self)?,
                NextResult::Return(offset) => {
                    let value = self.pop();
                    self.stack.truncate(offset);
                    self.push(value);
                }
                NextResult::Done => return Ok(self.pop()),
            };
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
    fn top(&self) -> &Frame {
        self.frames.last().unwrap()
    }
    fn top_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().unwrap()
    }

    pub fn send_direct(
        &mut self,
        handler: Rc<Handler>,
        target: Value,
        arity: usize,
    ) -> Runtime<()> {
        let local_offset = self.stack.len();
        for (i, param) in handler.params.iter().enumerate() {
            param.check_arg(&self.stack[local_offset - arity + i])?;
        }
        match target {
            Value::DoObject(_, return_from_index, ref self_value) => {
                self.frames.push(Frame::Handler {
                    handler,
                    ip: 0,
                    local_offset: local_offset - arity,
                    self_value: *self_value.clone(),
                    target_value: target,
                    return_from_index,
                })
            }
            _ => {
                let return_from_index = self.frames.len();
                self.frames.push(Frame::Handler {
                    handler,
                    ip: 0,
                    local_offset: local_offset - arity,
                    target_value: target.clone(),
                    self_value: target,
                    return_from_index,
                })
            }
        };
        Ok(())
    }

    pub fn send(&mut self, selector: &str, target: Value, arity: usize) -> Runtime<()> {
        let class = target.class();
        let handler = class.get(selector)?;
        let local_offset = self.stack.len();
        for (i, param) in handler.params.iter().enumerate() {
            param.check_arg(&self.stack[local_offset - arity + i])?;
        }
        match target {
            Value::DoObject(_, return_from_index, ref self_value) => {
                self.frames.push(Frame::Handler {
                    handler,
                    ip: 0,
                    local_offset: local_offset - arity,
                    self_value: *self_value.clone(),
                    target_value: target,
                    return_from_index,
                })
            }
            _ => {
                let return_from_index = self.frames.len();
                self.frames.push(Frame::Handler {
                    handler,
                    ip: 0,
                    local_offset: local_offset - arity,
                    self_value: target.clone(),
                    target_value: target,
                    return_from_index,
                })
            }
        };
        Ok(())
    }
    pub fn load_module(&mut self, module: &str) -> Runtime<Value> {
        self.modules.load(module)
    }
    pub fn get_stack(&mut self, address: Address) -> Value {
        self.stack[address].clone()
    }
    pub fn deref_pointer(&self, pointer: Value) -> Value {
        self.stack[pointer.as_pointer()].clone()
    }
    pub fn set_pointer(&mut self, pointer: Value, value: Value) {
        self.stack[pointer.as_pointer()] = value
    }
    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }
    pub fn take(&mut self, count: usize) -> Vec<Value> {
        self.stack.split_off(self.stack.len() - count)
    }
    pub fn local_offset(&self) -> usize {
        self.top().local_offset()
    }
    pub fn self_value(&self) -> Value {
        self.top().self_value()
    }
    pub fn get_ival(&self, index: usize) -> Value {
        self.top().ival(index)
    }
    pub fn return_from_index(&self) -> usize {
        self.top().return_from_index()
    }
    pub fn do_return(&mut self) {
        self.next_state = NextState::Return
    }
    pub fn do_loop(&mut self) {
        self.top_mut().do_loop()
    }
}

#[cfg(test)]
mod test {
    use crate::ir::{Class, Object, Param};

    use super::*;

    fn assert_ok(code: Vec<IR>, expected: Value) {
        let mut modules = ModuleLoader::new();
        assert_eq!(Interpreter::program(code, &mut modules), Ok(expected));
    }

    fn assert_err(code: Vec<IR>, expected: RuntimeError) {
        let mut modules = ModuleLoader::new();
        assert_eq!(Interpreter::program(code, &mut modules), Err(expected));
    }

    fn add() -> IR {
        IR::send("+:", 1)
    }

    #[test]
    fn addition() {
        assert_ok(vec![IR::int(1), IR::int(2), add()], Value::Integer(3))
    }

    #[test]
    fn does_not_understand() {
        assert_err(
            vec![IR::int(1), IR::send("foobar", 0)],
            RuntimeError::DoesNotUnderstand("foobar".to_string()),
        )
    }

    #[test]
    fn locals() {
        assert_ok(
            vec![
                IR::int(123), // 0
                IR::int(456), // 1
                IR::int(789), // 2
                IR::Local(1),
                IR::int(10),
                add(),
            ],
            Value::Integer(466),
        )
    }

    #[test]
    fn variables() {
        assert_ok(
            vec![
                IR::int(123), // 0
                IR::int(1),
                IR::Local(0),
                add(),
                IR::Var(0),
                IR::SetVar,
                IR::Local(0),
            ],
            Value::Integer(124),
        )
    }

    #[test]
    fn object_does_not_understand() {
        let empty_class = Class::new().rc();
        assert_err(
            vec![IR::object(empty_class, 0), IR::send("foobar", 0)],
            RuntimeError::DoesNotUnderstand("foobar".to_string()),
        )
    }

    #[test]
    fn simple_handlers() {
        let record = {
            let mut class = Class::new();
            class.add("x", vec![], vec![IR::int(1)]);
            class.add("y", vec![], vec![IR::int(2)]);
            class.rc()
        };
        assert_ok(
            vec![
                IR::object(record, 0), // 0
                IR::Local(0),
                IR::send("x", 0),
                IR::Local(0),
                IR::send("y", 0),
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
                    IR::int(123), // 0,
                    IR::int(100), // 1,
                    IR::Local(0),
                    IR::Local(1),
                    add(),
                ],
            );
            class.rc()
        };
        assert_ok(
            vec![
                IR::int(69),        // 0
                IR::int(420),       // 1
                IR::object(foo, 0), // 2
                IR::Local(2),
                IR::send("foo", 0),
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
                    IR::int(123), // 0,
                    IR::int(100), // 1,
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
                IR::int(69),        // 0
                IR::int(420),       // 1
                IR::object(foo, 0), // 2
                IR::Local(2),
                IR::send("foo", 0),
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
                    IR::int(10),
                    add(), // return $1 + 10
                ],
            );

            class.rc()
        };
        assert_ok(
            vec![
                IR::int(50),                       // $0
                IR::object(double_then_add_10, 0), // $1
                IR::Local(0),
                IR::Local(1),
                IR::send("foo:", 1), // $1{foo: $0}
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
                    IR::int(10),
                    add(), // return $1 + 10
                ],
            );

            class.rc()
        };

        assert_ok(
            vec![
                IR::int(50),                       // $0
                IR::object(double_then_add_10, 0), // $1
                IR::Local(0),
                IR::Local(1),
                IR::send("foo:", 1), // $2 = $1{foo: $0}
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
                    IR::int(10),
                    IR::Local(0),
                    IR::Deref,
                    add(),
                    IR::Local(0),
                    IR::SetVar, // $0 = *$0 + 10
                    IR::int(0), // (ignore return value)
                ],
            );

            class.rc()
        };

        assert_ok(
            vec![
                IR::object(add_10_to_var_arg, 0), // $0
                IR::int(100),                     // $1
                IR::Var(1),
                IR::Local(0),
                IR::send("foo:", 1), // $0{foo: var $1}
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
            class.rc()
        };

        assert_ok(
            vec![
                IR::int(1),
                IR::int(2),
                IR::object(pair.clone(), 2), // $0 = [x: 1 y: 2]
                IR::int(3),
                IR::int(4),
                IR::object(pair, 2), // $1 = [x: 3 y: 4]
                IR::Local(0),
                IR::send("x", 0),
                IR::Local(1),
                IR::send("y", 0),
                add(), // $0{x} + $1{y}
            ],
            Value::Integer(5),
        )
    }

    #[test]
    fn set_instance_value_var() {
        assert_ok(
            vec![
                IR::int(100), // $0
                IR::Var(0),
                IR::object(
                    {
                        let mut class = Class::new();
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
                                IR::int(0),
                            ],
                        );
                        class.rc()
                    },
                    1,
                ), // $1
                IR::int(20),
                IR::Local(1),
                IR::send("add to var:", 1),
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
                    IR::int(10),
                    IR::Local(0),
                    add(),
                    IR::Return,
                    // should be unreachable
                    IR::int(20),
                ],
            );
            class.rc()
        };
        assert_ok(
            vec![IR::int(3), IR::object(obj, 0), IR::send("add 10:", 1)],
            Value::Integer(13),
        );
    }

    #[test]
    fn return_from_root() {
        assert_ok(
            vec![
                IR::int(3),
                IR::Return,
                // should be unreachable
                IR::int(4),
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
                IR::object(
                    {
                        let mut class = Class::new();
                        class.add(
                            "run",
                            vec![],
                            vec![
                                IR::object(
                                    {
                                        let mut class = Class::new();
                                        class.add(
                                            "match:",
                                            vec![Param::Do],
                                            vec![
                                                IR::int(50),
                                                IR::Local(0),
                                                IR::send("some:", 1),
                                                // unreachable if do block returns early
                                                IR::int(456),
                                            ],
                                        );
                                        class.rc()
                                    },
                                    0,
                                ), // $0
                                IR::DoObject(
                                    {
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
                                                IR::int(123),
                                            ],
                                        );
                                        class.rc()
                                    },
                                    0,
                                ),
                                IR::Local(0),
                                IR::send("match:", 1),
                                // unreachable if match do arg returns early
                                IR::int(789),
                            ],
                        );
                        class.rc()
                    },
                    0,
                ),
                IR::send("run", 0),
            ],
            Value::Integer(100),
        );
    }

    #[test]
    fn arg_type_error() {
        assert_err(
            vec![
                IR::int(1),
                IR::object(
                    {
                        let mut class = Class::new();
                        class.add("foo:", vec![Param::Var], vec![IR::unit()]);
                        class.rc()
                    },
                    0,
                ),
                IR::send("foo:", 1),
            ],
            RuntimeError::ExpectedVarArg,
        );
        assert_err(
            vec![
                IR::DoObject(Class::new().rc(), 0),
                IR::object(
                    {
                        let mut class = Class::new();
                        class.add("foo:", vec![Param::Value], vec![IR::unit()]);
                        class.rc()
                    },
                    0,
                ),
                IR::send("foo:", 1),
            ],
            RuntimeError::DidNotExpectDoArg,
        );
    }

    #[test]
    fn new_self() {
        let class = {
            let mut class = Class::new();
            class.add(
                "value:",
                vec![Param::Value],
                vec![IR::Local(0), IR::NewSelf(1)],
            );
            class.rc()
        };

        assert_ok(
            vec![
                IR::int(123),
                IR::object(class.clone(), 1),
                IR::int(456),
                IR::Local(0),
                IR::send("value:", 1),
            ],
            Value::Object(Object::new(class, vec![Value::Integer(456)]).rc()),
        )
    }

    #[test]
    fn self_ref() {
        let class = {
            let mut class = Class::new();
            class.add("x", vec![], vec![IR::int(123)]);
            class.add("x1", vec![], vec![IR::SelfRef, IR::send("x", 0)]);
            class.rc()
        };

        assert_ok(
            vec![IR::object(class, 0), IR::send("x1", 0)],
            Value::Integer(123),
        )
    }

    #[test]
    fn native_fn() {
        assert_ok(
            vec![
                IR::int(2),
                IR::SendNative(|x, _| Ok(Value::Integer(x.as_int() << 2)), 0),
            ],
            Value::Integer(8),
        )
    }

    #[test]
    fn modules() {
        let mut modules = ModuleLoader::new();
        modules.add_ready("foo", Value::Integer(123));

        assert_eq!(
            Interpreter::program(vec![IR::Module("foo".to_string())], &mut modules),
            Ok(Value::Integer(123))
        );
    }

    #[test]
    fn module_load_loop() {
        let mut modules = ModuleLoader::new();
        modules.add_init("foo", vec![IR::Module("foo".to_string())]);

        assert_eq!(
            Interpreter::program(vec![IR::Module("foo".to_string())], &mut modules),
            Err(RuntimeError::ModuleLoadLoop("foo".to_string()))
        );
    }

    #[test]
    fn unknown_module() {
        assert_err(
            vec![IR::Module("unknown".to_string())],
            RuntimeError::UnknownModule("unknown".to_string()),
        )
    }

    #[test]
    fn try_send() {
        assert_ok(
            vec![
                IR::DoObject(
                    {
                        let mut class = Class::new();
                        class.add("", vec![], vec![IR::int(123)]);
                        class.rc()
                    },
                    0,
                ),
                IR::int(1),
                IR::TrySend("unknown".to_string(), 0),
            ],
            Value::Integer(123),
        )
    }
}
