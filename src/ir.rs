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
            IR::Unit => ctx.stack.push(Value::Unit),
            IR::MutArray => ctx
                .stack
                .push(Value::MutArray(Rc::new(RefCell::new(Vec::new())))),
            IR::SelfRef => ctx.stack.push(ctx.call_stack.self_value()),
            IR::Bool(value) => ctx.stack.push(Value::Bool(value)),
            IR::SendBool => {
                let bool = ctx.stack.pop().as_bool();
                let target = ctx.stack.pop();
                if bool {
                    target.send("true", 0, &mut ctx.stack, &mut ctx.call_stack)?;
                } else {
                    target.send("false", 0, &mut ctx.stack, &mut ctx.call_stack)?;
                }
            }
            IR::Integer(value) => ctx.stack.push(Value::Integer(value)),
            IR::String(str) => ctx.stack.push(Value::String(str)),
            IR::Local(address) => {
                let local_offset = ctx.call_stack.local_offset();
                let value = ctx.stack.get(address + local_offset);
                ctx.stack.push(value);
            }
            IR::IVal(index) => {
                let value = ctx.call_stack.ival(index);
                ctx.stack.push(value);
            }
            IR::Var(address) => {
                let absolute_address = address + ctx.call_stack.local_offset();
                ctx.stack.push(Value::Pointer(absolute_address));
            }
            IR::Object(class, arity) => {
                let ivals = Rc::new(ctx.stack.take(arity));
                let value = Value::Object(class, ivals);
                ctx.stack.push(value);
            }
            IR::NewSelf(arity) => {
                let class = ctx.call_stack.self_value().class();
                let ivals = Rc::new(ctx.stack.take(arity));
                let value = Value::Object(class, ivals);
                ctx.stack.push(value);
            }
            IR::DoObject(class, arity) => {
                let ivals = Rc::new(ctx.stack.take(arity));
                let return_from_index = ctx.call_stack.return_from_index();
                let self_value = Box::new(ctx.call_stack.self_value());
                let value = Value::DoObject(class, ivals, return_from_index, self_value);
                ctx.stack.push(value);
            }
            IR::Module(name) => {
                let value = ctx.modules.load(&name)?;
                ctx.stack.push(value);
            }
            IR::Deref => {
                let pointer = ctx.stack.pop();
                let value = pointer.deref(&ctx.stack);
                ctx.stack.push(value);
            }
            IR::SetVar => {
                let pointer = ctx.stack.pop();
                let value = ctx.stack.pop();
                pointer.set(value, &mut ctx.stack);
            }
            IR::Send(selector, arity) => {
                let target = ctx.stack.pop();
                target.send(&selector, arity, &mut ctx.stack, &mut ctx.call_stack)?;
            }
            IR::TrySend(selector, arity) => {
                let target = ctx.stack.pop();
                let or_else = ctx.stack.pop();
                match target.send(&selector, arity, &mut ctx.stack, &mut ctx.call_stack) {
                    Ok(_) => {}
                    Err(_) => {
                        ctx.stack.take(arity);
                        or_else.send("", 0, &mut ctx.stack, &mut ctx.call_stack)?;
                    }
                }
            }
            IR::SendNative(f, arity) => {
                let target = ctx.stack.pop();
                let args = ctx.stack.take(arity);
                let result = f(target, args)?;
                ctx.stack.push(result);
            }
            IR::Return => ctx.call_stack.do_return(),
            IR::Loop => ctx.call_stack.do_loop(),
            IR::Drop => {
                ctx.stack.pop();
            }
        }
        Ok(())
    }
}
