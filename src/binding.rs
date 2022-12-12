use std::collections::HashMap;

use crate::{
    compiler::{CompileIR, Compiler},
    ir::IR,
    source::Source,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Binding {
    Identifier(String, Source),
    Placeholder(Source),
    Destructuring(HashMap<String, Binding>, Source),
}

impl Binding {
    // value being bound is on top of IR stack
    pub fn compile_let(self, compiler: &mut Compiler, is_export: bool) -> CompileIR {
        match self {
            Binding::Identifier(name, _) => {
                let index = compiler.add_let(name.to_string());
                if is_export {
                    compiler.export(&name, index)?;
                }
                Ok(vec![])
            }
            Binding::Placeholder(_) => {
                return Ok(vec![IR::Drop]);
            }
            Binding::Destructuring(map, _) => {
                let mut out = vec![];
                let index = compiler.add_anon();
                for (key, binding) in map {
                    out.push(IR::Local { index });
                    out.push(IR::Send {
                        selector: key,
                        arity: 0,
                    });
                    let mut child = binding.compile_let(compiler, is_export)?;
                    out.append(&mut child);
                }
                return Ok(out);
            }
        }
    }
    pub fn bind_param(self, compiler: &mut Compiler) -> BindParamResult {
        match self {
            Binding::Identifier(key, _) => {
                compiler.add_let(key);
                BindParamResult::None
            }
            Binding::Placeholder(_) => {
                compiler.add_anon();
                BindParamResult::None
            }
            Binding::Destructuring(map, _) => {
                let index = compiler.add_anon();
                BindParamResult::Destructuring(map, index)
            }
        }
    }
}

#[derive(Debug)]
pub enum BindParamResult {
    None,
    Destructuring(HashMap<String, Binding>, usize),
}

impl BindParamResult {
    pub fn compile(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            BindParamResult::None => Ok(vec![]),
            BindParamResult::Destructuring(map, index) => {
                let mut out = vec![];
                for (key, binding) in map {
                    out.push(IR::Local { index });
                    out.push(IR::Send {
                        selector: key,
                        arity: 0,
                    });
                    let mut child = binding.compile_let(compiler, false)?;
                    out.append(&mut child);
                }
                Ok(out)
            }
        }
    }
}
