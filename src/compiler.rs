use std::collections::HashMap;

use crate::{object_builder::Exports, parse_stmt::Stmt, runtime::IR, value::Value};

#[derive(Debug, PartialEq, Clone)]
pub enum CompileError {
    UnknownIdentifier(String),
    InvalidSelf,
    InvalidVarBinding,
}

pub type CompileIR = Result<Vec<IR>, CompileError>;
pub type Compile<T> = Result<T, CompileError>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BindingType {
    Let,
    Var,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct BindingRecord {
    pub index: usize,
    pub typ: BindingType,
}

#[derive(Debug)]
struct Locals {
    index: usize,
    map: HashMap<String, BindingRecord>,
}

impl Locals {
    fn root() -> Self {
        Self {
            index: 0,
            map: HashMap::new(),
        }
    }
    fn get(&self, key: &str) -> Option<BindingRecord> {
        self.map.get(key).map(|r| r.to_owned())
    }
    fn add(&mut self, key: String, typ: BindingType) -> BindingRecord {
        let record = BindingRecord {
            index: self.index,
            typ,
        };
        self.index += 1;
        self.map.insert(key, record);
        record
    }
    fn add_anon(&mut self, typ: BindingType) -> BindingRecord {
        let record = BindingRecord {
            index: self.index,
            typ,
        };
        self.index += 1;
        record
    }
}

#[derive(Debug)]
pub struct Instance {
    ivars: Vec<IR>,
    ivar_map: HashMap<String, BindingRecord>,
}

impl Instance {
    pub fn new() -> Self {
        Self {
            ivars: Vec::new(),
            ivar_map: HashMap::new(),
        }
    }
    fn get(&self, key: &str) -> Option<IR> {
        if let Some(record) = self.ivar_map.get(key) {
            return Some(IR::IVar {
                index: record.index,
            });
        }
        return None;
    }
    fn add(&mut self, key: &str, value: IR) -> IR {
        let index = self.ivars.len();
        self.ivars.push(value);
        self.ivar_map.insert(
            key.to_string(),
            BindingRecord {
                index,
                typ: BindingType::Let,
            },
        );
        return IR::IVar { index };
    }
    pub fn ivars(self) -> Vec<IR> {
        self.ivars
    }
}

#[derive(Debug)]
enum Scope {
    Root,
    Handler(Instance),
}

#[derive(Debug)]
struct CompilerFrame {
    locals: Locals,
    scope: Scope,
}

impl CompilerFrame {
    fn root() -> Self {
        Self {
            locals: Locals::root(),
            scope: Scope::Root,
        }
    }
    fn handler(instance: Instance) -> Self {
        Self {
            locals: Locals::root(),
            scope: Scope::Handler(instance),
        }
    }
    fn add(&mut self, key: String, typ: BindingType) -> BindingRecord {
        self.locals.add(key, typ)
    }
    fn add_anon(&mut self, typ: BindingType) -> BindingRecord {
        self.locals.add_anon(typ)
    }
}

#[derive(Debug)]
pub struct Compiler {
    stack: Vec<CompilerFrame>,
    exports: Exports,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            stack: vec![CompilerFrame::root()],
            exports: Exports::new(),
        }
    }
    pub fn program(program: Vec<Stmt>) -> CompileIR {
        let mut compiler = Self::new();
        Compiler::body(program, &mut compiler)
    }

    pub fn module(program: Vec<Stmt>) -> CompileIR {
        let mut compiler = Self::new();
        let mut out = Compiler::body(program, &mut compiler)?;
        let mut exports = compiler.exports.compile()?;
        out.append(&mut exports);
        Ok(out)
    }

    pub fn body(mut body: Vec<Stmt>, compiler: &mut Compiler) -> CompileIR {
        if let Some(last) = body.pop() {
            let mut out = Vec::new();
            for stmt in body {
                let mut res = stmt.compile(compiler)?;
                out.append(&mut res);
            }
            match last {
                Stmt::Expr(expr) => {
                    let mut res = expr.compile(compiler)?;
                    out.append(&mut res);
                }
                stmt => {
                    let mut res = stmt.compile(compiler)?;
                    out.append(&mut res);
                    out.push(IR::Constant(Value::Unit));
                }
            }
            Ok(out)
        } else {
            Ok(vec![IR::Constant(Value::Unit)])
        }
    }

    pub fn export(&mut self, name: &str, record: BindingRecord) -> Compile<()> {
        self.exports.add(name, record.index)
    }

    pub fn handler(&mut self, instance: Instance) {
        self.stack.push(CompilerFrame::handler(instance));
    }
    pub fn end_handler(&mut self) -> Instance {
        let frame = self.stack.pop().expect("compiler frame underflow");
        match frame.scope {
            Scope::Handler(instance) => instance,
            _ => panic!("expected handler frame"),
        }
    }

    pub fn add_anon(&mut self) -> BindingRecord {
        self.top_mut().add_anon(BindingType::Let)
    }
    pub fn add_let(&mut self, key: String) -> BindingRecord {
        self.top_mut().add(key, BindingType::Let)
    }
    pub fn add_var(&mut self, key: String) -> BindingRecord {
        self.top_mut().add(key, BindingType::Var)
    }
    fn top(&self) -> &CompilerFrame {
        self.stack.last().unwrap()
    }
    fn top_mut(&mut self) -> &mut CompilerFrame {
        self.stack.last_mut().unwrap()
    }
    pub fn get_self(&self) -> CompileIR {
        match self.top().scope {
            Scope::Root => Err(CompileError::InvalidSelf),
            _ => Ok(vec![IR::SelfRef]),
        }
    }
    pub fn get(&mut self, key: &str) -> CompileIR {
        let frame = self.top();
        if let Some(value) = frame.locals.get(key) {
            return Ok(vec![IR::Local { index: value.index }]);
        }
        if let Scope::Handler(instance) = &frame.scope {
            if let Some(ir) = instance.get(key) {
                return Ok(vec![ir]);
            }
        }

        let next_depth = self.stack.len() - 2;
        self.get_parent(key, next_depth)
    }

    pub fn get_parent(&mut self, key: &str, depth: usize) -> CompileIR {
        let frame = &mut self.stack[depth];

        if let Scope::Handler(instance) = &frame.scope {
            if let Some(ir) = instance.get(key) {
                return Ok(vec![self.get_found(ir, key, depth)]);
            }
        }
        if let Some(value) = frame.locals.get(key) {
            match value.typ {
                BindingType::Let => {
                    let ir = IR::Local { index: value.index };
                    return Ok(vec![self.get_found(ir, key, depth)]);
                }
                BindingType::Var => return Err(CompileError::InvalidVarBinding),
            }
        }
        if depth == 0 {
            return Err(CompileError::UnknownIdentifier(key.to_string()));
        }
        self.get_parent(key, depth - 1)
    }

    fn get_found(&mut self, ir: IR, key: &str, index: usize) -> IR {
        let mut out = ir;
        for i in (index + 1)..self.stack.len() {
            if let Scope::Handler(instance) = &mut self.stack[i].scope {
                out = instance.add(key, out);
            }
        }
        out
    }
    pub fn get_var_index(&self, key: &str) -> Option<usize> {
        self.top()
            .locals
            .get(key)
            .and_then(|record| match record.typ {
                BindingType::Let => None,
                BindingType::Var => Some(record.index),
            })
    }
}

#[cfg(test)]
mod test {
    // use crate::{
    //     class::{Class, Param},
    //     value::Value,
    // };

    use super::*;

    fn compile(code: &str) -> CompileIR {
        let lexer = crate::lexer::Lexer::from_string(code);
        let mut parser = crate::parser::Parser::new(lexer);
        let program = parser.program().unwrap();
        Compiler::program(program)
    }

    // fn assert_ok(code: &str, expected: Vec<IR>) {
    //     assert_eq!(compile(code), Ok(expected));
    // }

    fn assert_err(code: &str, err: CompileError) {
        assert_eq!(compile(code), Err(err));
    }

    #[test]
    fn vars() {
        assert_err(
            "
            var x := 1
            let obj := [
                on {x} x
            ]
        ",
            CompileError::InvalidVarBinding,
        )
    }
}
