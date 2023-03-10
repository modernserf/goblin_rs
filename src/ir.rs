use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::native::{array_class, big_int_class, bool_class, int_class, string_class, unit_class};
use crate::runtime::{Interpreter, Runtime, RuntimeError};

pub type Address = usize;
pub type Selector = String;
pub type Index = usize;
pub type Arity = usize;
pub type NativeFn = fn(Value, Vec<Value>) -> Runtime<Value>;

type MoreFnInner = fn(&mut Interpreter) -> Runtime<()>;

#[derive(Clone)]
pub struct MoreFn(MoreFnInner);
impl std::fmt::Debug for MoreFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<more fn>")
    }
}
impl PartialEq for MoreFn {
    fn eq(&self, other: &Self) -> bool {
        self.0 as usize == other.0 as usize
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Constant(Value),                // ( -- value)
    Local(Address),                 // ( -- *address)
    Var(Address),                   // ( -- address)
    IVal(Index),                    // ( -- instance[index])
    SelfRef,                        // ( -- self_value)
    Module(String),                 // ( -- module)
    Object(Rc<Class>, Arity),       // (...instance -- object)
    DoObject(Rc<Class>, Arity),     // (...instance -- object)
    NewSelf(Arity),                 // (...instance -- object)
    Deref,                          // (address -- *address)
    SetVar,                         // (value address -- )
    Send(Selector, Arity),          // (...args target -- result)
    SendDirect(Rc<Handler>, Arity), // (...args target -- result)
    TrySend(Selector, Arity),       // (...args target -- result)
    SendNative(NativeFn, Arity),    // (...args target -- result)
    Native(MoreFn),                 // (...)
    Drop,                           // (value --)
    Return,
    Loop,
}

impl IR {
    pub fn unit() -> Self {
        IR::Constant(Value::Unit)
    }
    pub fn int(value: i64) -> Self {
        IR::Constant(Value::Integer(value))
    }
    pub fn bool(value: bool) -> Self {
        IR::Constant(Value::Bool(value))
    }
    pub fn string(value: String) -> Self {
        IR::Constant(Value::String(Rc::new(value)))
    }
    pub fn send(selector: &str, arity: usize) -> Self {
        IR::Send(selector.to_string(), arity)
    }
    pub fn native(f: MoreFnInner) -> Self {
        IR::Native(MoreFn(f))
    }
    pub fn object(class: Rc<Class>, arity: usize) -> Self {
        if arity == 0 {
            IR::Constant(Value::Object(Object::new(class, vec![]).rc()))
        } else {
            IR::Object(class, arity)
        }
    }

    pub fn eval(self, ctx: &mut Interpreter) -> Runtime<()> {
        match self {
            IR::Constant(value) => ctx.push(value),
            IR::Native(f) => return f.0(ctx),
            IR::SelfRef => {
                let value = ctx.self_value();
                ctx.push(value)
            }
            IR::Local(address) => {
                let local_offset = ctx.local_offset();
                let value = ctx.get_stack(address + local_offset);
                ctx.push(value);
            }
            IR::IVal(index) => {
                let value = ctx.get_ival(index);
                ctx.push(value);
            }
            IR::Var(address) => {
                let absolute_address = address + ctx.local_offset();
                ctx.push(Value::Pointer(absolute_address));
            }
            IR::Object(class, arity) => {
                let ivals = ctx.take(arity);
                let value = Value::Object(Object::new(class, ivals).rc());
                ctx.push(value);
            }
            IR::NewSelf(arity) => {
                let class = match ctx.self_value() {
                    Value::Object(obj) => obj.class.clone(),
                    _ => panic!("cannot get class"),
                };
                let ivals = ctx.take(arity);
                let value = Value::Object(Object::new(class, ivals).rc());
                ctx.push(value);
            }
            IR::DoObject(class, arity) => {
                let ivals = ctx.take(arity);
                let return_from_index = ctx.return_from_index();
                let self_value = Box::new(ctx.self_value());
                let value = Value::DoObject(
                    Object::new(class, ivals).rc(),
                    return_from_index,
                    self_value,
                );
                ctx.push(value);
            }
            IR::Module(name) => {
                let value = ctx.load_module(&name)?;
                ctx.push(value);
            }
            IR::Deref => {
                let pointer = ctx.pop();
                let value = ctx.deref_pointer(pointer);
                ctx.push(value);
            }
            IR::SetVar => {
                let pointer = ctx.pop();
                let value = ctx.pop();
                ctx.set_pointer(pointer, value);
            }
            IR::Send(selector, arity) => {
                let target = ctx.pop();
                ctx.send(&selector, target, arity)?;
            }
            IR::SendDirect(handler, arity) => {
                let target = ctx.pop();
                ctx.send_direct(handler, target, arity)?;
            }
            IR::TrySend(selector, arity) => {
                let target = ctx.pop();
                let or_else = ctx.pop();
                match ctx.send(&selector, target, arity) {
                    Ok(_) => {}
                    Err(_) => {
                        ctx.take(arity);
                        ctx.send("", or_else, 0)?;
                    }
                }
            }
            IR::SendNative(f, arity) => {
                let target = ctx.pop();
                let args = ctx.take(arity);
                let result = f(target, args)?;
                ctx.push(result);
            }
            IR::Return => ctx.do_return(),
            IR::Loop => ctx.do_loop(),
            IR::Drop => {
                ctx.pop();
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),
    Integer(i64),
    // TODO
    Bigint(u128),
    String(Rc<String>),
    Object(Rc<Object>),
    DoObject(Rc<Object>, ParentFrameIndex, Box<Value>),
    Pointer(Address),
    MutArray(MutArray),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutArray {
    value: Rc<RefCell<Vec<Value>>>,
}

impl MutArray {
    fn debug(&self) -> String {
        self.value
            .borrow_mut()
            .iter()
            .map(|x| x.debug())
            .collect::<Vec<_>>()
            .join(",")
    }
}

// impl PartialEq for MutArray {
//     fn eq(&self, other: &Self) -> bool {
//         Rc::ptr_eq(&self.value, &other.value)
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub class: Rc<Class>,
    pub ivals: Vec<Value>,
}

impl Object {
    pub fn new(class: Rc<Class>, ivals: Vec<Value>) -> Self {
        Object { class, ivals }
    }
    pub fn rc(self) -> Rc<Self> {
        Rc::new(self)
    }
}

pub type ParentFrameIndex = usize;

impl Value {
    pub fn mut_array(vec: Vec<Value>) -> Self {
        Value::MutArray(MutArray {
            value: Rc::new(RefCell::new(vec)),
        })
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(val) => *val,
            _ => panic!("cannot cast to bool"),
        }
    }
    pub fn as_int(&self) -> i64 {
        match self {
            Value::Integer(val) => *val,
            _ => panic!("cannot cast to int"),
        }
    }
    pub fn as_string(&self) -> Rc<String> {
        match self {
            Value::String(str) => str.clone(),
            _ => panic!("cannot cast to string"),
        }
    }
    pub fn as_array(&self) -> Rc<RefCell<Vec<Value>>> {
        match self {
            Value::MutArray(arr) => arr.value.clone(),
            _ => panic!("cannot cast to array"),
        }
    }
    pub fn as_bigint(&self) -> u128 {
        match self {
            Value::Bigint(val) => *val,
            _ => panic!("cannot cast to bigint"),
        }
    }
    pub fn as_pointer(&self) -> usize {
        match self {
            Value::Pointer(address) => *address,
            _ => panic!("deref a non-pointer"),
        }
    }

    pub fn debug(&self) -> String {
        match self {
            Value::Pointer(_) => panic!("must deref pointer before sending message"),
            Value::Unit => "()".to_string(),
            Value::Integer(value) => value.to_string(),
            Value::Bigint(value) => value.to_string(),
            Value::String(value) => format!("\"{}\"", value),
            Value::Bool(value) => value.to_string(),
            Value::MutArray(items) => items.debug(),
            Value::Object(obj) => format!(
                "{{{}}}",
                obj.ivals
                    .iter()
                    .map(|val| val.debug())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            Value::DoObject(_, _, _) => "<do object>".to_string(),
        }
    }

    pub fn class(&self) -> Rc<Class> {
        match self {
            Value::Pointer(_) => panic!("must deref pointer before sending message"),
            Value::Unit => unit_class(),
            Value::Integer(_) => int_class(),
            Value::Bigint(_) => big_int_class(),
            Value::String(_) => string_class(),
            Value::Bool(_) => bool_class(),
            Value::MutArray(_) => array_class(),
            Value::Object(obj) => obj.class.clone(),
            Value::DoObject(obj, _, _) => obj.class.clone(),
        }
    }
    pub fn ival(&self, index: Index) -> Value {
        match self {
            Value::Object(obj) => obj.ivals[index].clone(),
            Value::DoObject(obj, _, _) => obj.ivals[index].clone(),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    handlers: HashMap<Selector, Rc<Handler>>,
}

impl Class {
    pub fn new() -> Self {
        Class {
            handlers: HashMap::new(),
        }
    }
    pub fn add(&mut self, selector: &str, params: Vec<Param>, body: Vec<IR>) {
        self.add_handler(selector.to_string(), params, body)
    }
    pub fn add_handler(&mut self, selector: String, params: Vec<Param>, body: Vec<IR>) {
        self.handlers.insert(
            selector.to_string(),
            Rc::new(Handler {
                selector,
                body,
                params,
            }),
        );
    }
    pub fn add_native(&mut self, selector: &str, params: Vec<Param>, f: NativeFn) {
        let arity = params.len();
        self.add_handler(
            selector.to_string(),
            params,
            vec![IR::SelfRef, IR::SendNative(f, arity)],
        );
    }
    pub fn get(&self, selector: &str) -> Runtime<Rc<Handler>> {
        match self.handlers.get(selector) {
            Some(handler) => Ok(handler.clone()),
            None => Err(RuntimeError::DoesNotUnderstand(selector.to_string())),
        }
    }
    pub fn rc(self) -> Rc<Class> {
        Rc::new(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Handler {
    pub selector: String,
    pub params: Vec<Param>,
    pub body: Vec<IR>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Param {
    Value,
    Var,
    Do,
}
impl Param {
    pub fn check_arg(&self, arg: &Value) -> Runtime<()> {
        match (self, arg) {
            (Param::Var, Value::Pointer(_)) => Ok(()),
            (Param::Var, _) => Err(RuntimeError::ExpectedVarArg),
            (Param::Do, Value::DoObject(..)) => Ok(()),
            (_, Value::DoObject(..)) => Err(RuntimeError::DidNotExpectDoArg),
            _ => Ok(()),
        }
    }
}
