use std::collections::HashMap;

use crate::{
    interpreter::{program, SendEffect},
    ir::IR,
    runtime_error::RuntimeError,
    value::Value,
};

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
    pub fn load(&mut self, name: &str) -> SendEffect {
        match self.modules.get_mut(name) {
            Some(ModuleLoadState::Loading) => {
                todo!("error: loop in load module")
            }
            Some(ModuleLoadState::Ready(value)) => value.clone().eval(),
            Some(ModuleLoadState::Init(ir)) => {
                let ir = std::mem::take(ir);
                self.modules
                    .insert(name.to_string(), ModuleLoadState::Loading);

                match program(ir, self) {
                    Ok(value) => {
                        self.add_ready(name, value.clone());
                        SendEffect::Value(value)
                    }
                    Err(err) => SendEffect::Error(err),
                }
            }
            None => RuntimeError::unknown_module(&name),
        }
    }
}
