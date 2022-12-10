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
                let record = compiler.add_let(name.to_string());
                if is_export {
                    compiler.export(&name, record)?;
                }
                return Ok(vec![IR::Assign(record.index)]);
            }
            Binding::Placeholder(_) => {
                return Ok(vec![IR::Drop]);
            }
            Binding::Destructuring(map, _) => {
                let mut out = vec![];
                let root_record = compiler.add_anon();
                out.push(IR::Assign(root_record.index));
                for (key, binding) in map {
                    out.push(IR::Local(root_record.index));
                    out.push(IR::Send(key, 0));
                    let mut child = binding.compile_let(compiler, is_export)?;
                    out.append(&mut child);
                }
                return Ok(out);
            }
        }
    }
}
