use std::{cell::RefCell, rc::Rc};

use crate::runtime::{Class, Interpreter, Runtime, Value};

pub type Address = usize;
pub type Selector = String;
pub type Index = usize;
pub type Arity = usize;
pub type NativeFn = fn(Value, Vec<Value>) -> Runtime<Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum IR {
    Unit,                        // (-- value)
    Bool(bool),                  // (-- value)
    Integer(i64),                // (-- value)
    String(Rc<String>),          // (-- value)
    MutArray,                    // (-- array)
    Local(Address),              // ( -- *address)
    Var(Address),                // ( -- address)
    IVal(Index),                 // ( -- instance[index])
    SelfRef,                     // ( -- self_value)
    Module(String),              // ( -- module)
    Object(Rc<Class>, Arity),    // (...instance -- object)
    DoObject(Rc<Class>, Arity),  // (...instance -- object)
    NewSelf(Arity),              // (...instance -- object)
    Deref,                       // (address -- *address)
    SetVar,                      // (value address -- )
    Send(Selector, Arity),       // (...args target -- result)
    TrySend(Selector, Arity),    // (...args target -- result)
    SendNative(NativeFn, Arity), // (...args target -- result)
    SendBool,                    // (target bool -- result)
    Drop,                        // (value --)
    Return,
    Loop,
}

impl IR {
    pub fn send(selector: &str, arity: usize) -> Self {
        Self::Send(selector.to_string(), arity)
    }

    pub fn eval(self, ctx: &mut Interpreter) -> Runtime<()> {
        match self {
            IR::Unit => ctx.push(Value::Unit),
            IR::MutArray => ctx.push(Value::MutArray(Rc::new(RefCell::new(Vec::new())))),
            IR::SelfRef => {
                let value = ctx.self_value();
                ctx.push(value)
            }
            IR::Bool(value) => ctx.push(Value::Bool(value)),
            IR::SendBool => {
                let bool = ctx.pop().as_bool();
                let target = ctx.pop();
                if bool {
                    ctx.send("true", target, 0)?;
                } else {
                    ctx.send("false", target, 0)?;
                }
            }
            IR::Integer(value) => ctx.push(Value::Integer(value)),
            IR::String(str) => ctx.push(Value::String(str)),
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
                let ivals = Rc::new(ctx.take(arity));
                let value = Value::Object(class, ivals);
                ctx.push(value);
            }
            IR::NewSelf(arity) => {
                let class = ctx.self_value().class();
                let ivals = Rc::new(ctx.take(arity));
                let value = Value::Object(class, ivals);
                ctx.push(value);
            }
            IR::DoObject(class, arity) => {
                let ivals = Rc::new(ctx.take(arity));
                let return_from_index = ctx.return_from_index();
                let self_value = Box::new(ctx.self_value());
                let value = Value::DoObject(class, ivals, return_from_index, self_value);
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
