use std::collections::HashMap;

use crate::{interpreter::SendEffect, ir::IR, runtime_error::RuntimeError, value::Value};

#[derive(Debug, Clone)]
enum ModuleLoadState {
    Init(Vec<IR>),
    Loading(Vec<IR>),
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
    pub fn add_init(&mut self, name: String, ir: Vec<IR>) {
        self.modules.insert(name, ModuleLoadState::Init(ir));
    }
    pub fn add_ready(&mut self, name: String, value: Value) {
        self.modules.insert(name, ModuleLoadState::Ready(value));
    }
    pub fn load(&mut self, name: &str) -> SendEffect {
        let module = self.modules.get(name);
        match module {
            Some(ModuleLoadState::Loading(_)) => {
                todo!("error: loop in load module")
            }
            Some(ModuleLoadState::Ready(value)) => value.clone().eval(),
            Some(ModuleLoadState::Init(ir)) => {
                todo!("run module to get exports")
            }
            None => RuntimeError::unknown_module(&name),
        }
    }
}
