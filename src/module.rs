use std::collections::HashMap;

use crate::ir::IR;
use crate::runtime::{eval_module, Runtime, RuntimeError};
use crate::value::Value;

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

                match eval_module(ir, self) {
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
