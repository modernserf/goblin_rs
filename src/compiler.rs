use std::collections::HashMap;

use crate::{class::Class, ir::IR, stmt::Stmt};

#[derive(Debug, PartialEq, Clone)]
pub enum CompileError {
    UnknownIdentifier(String),
    InvalidSelf,
    InvalidVarBinding,
}

pub type CompileIR = Result<Vec<IR>, CompileError>;
pub type Compile<T> = Result<T, CompileError>;

#[derive(Debug, Clone)]
pub struct Exports {
    exports: HashMap<String, usize>,
}

impl Exports {
    pub fn new() -> Self {
        Exports {
            exports: HashMap::new(),
        }
    }
    pub fn add(&mut self, key: &str, index: usize) -> Compile<()> {
        if self.exports.insert(key.to_string(), index).is_some() {
            todo!("error: duplicate export")
        }
        Ok(())
    }

    pub fn compile(self) -> CompileIR {
        let mut entries = self.exports.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));
        let mut out = Vec::new();
        let mut class = Class::new();
        for (ivar, (key, local)) in entries.iter().enumerate() {
            out.push(IR::Local { index: *local });
            class.add_handler(&key, vec![], vec![IR::IVar { index: ivar }]);
        }
        out.push(IR::NewObject {
            class: class.rc(),
            arity: entries.len(),
        });
        Ok(out)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum BindingType {
    Let,
    Var,
}
#[derive(Debug, PartialEq, Clone, Copy)]
struct BindingRecord {
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

#[derive(Debug, Clone)]
enum GetResult {
    Local(usize),
    Var(usize),
    IVar(usize),
    Parent(usize),
    ParentVar(usize),
}

impl GetResult {
    fn ir(self) -> IR {
        match self {
            Self::Local(index) => IR::Local { index },
            Self::Var(index) => IR::Local { index },
            Self::IVar(index) => IR::IVar { index },
            Self::Parent(index) => IR::Parent { index },
            Self::ParentVar(index) => IR::Parent { index },
        }
    }
    fn is_var(&self) -> bool {
        match self {
            Self::Var(_) | Self::ParentVar(_) => true,
            _ => false,
        }
    }
    fn assign_ir(self) -> IR {
        match self {
            Self::Var(index) => IR::SetLocal { index },
            Self::ParentVar(index) => IR::SetParent { index },
            _ => unreachable!(),
        }
    }
    fn parent(self) -> Self {
        match self {
            Self::Local(index) => Self::Parent(index),
            Self::Var(index) => Self::ParentVar(index),
            Self::IVar(index) => Self::IVar(index),
            Self::Parent(index) => Self::Parent(index),
            Self::ParentVar(index) => Self::ParentVar(index),
        }
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
    fn get(&self, key: &str) -> Option<GetResult> {
        if let Some(record) = self.ivar_map.get(key) {
            return Some(GetResult::IVar(record.index));
        }
        return None;
    }
    fn add(&mut self, key: &str, value: GetResult) -> GetResult {
        let index = self.ivars.len();
        self.ivars.push(value.ir());
        self.ivar_map.insert(
            key.to_string(),
            BindingRecord {
                index,
                typ: BindingType::Let,
            },
        );
        return GetResult::IVar(index);
    }
    pub fn ivars(self) -> Vec<IR> {
        self.ivars
    }
}

#[derive(Debug)]
enum Scope {
    Root,
    DoHandler,
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
    fn do_handler() -> Self {
        Self {
            locals: Locals::root(),
            scope: Scope::DoHandler,
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
                    out.push(IR::Unit);
                }
            }
            Ok(out)
        } else {
            Ok(vec![IR::Unit])
        }
    }

    pub fn export(&mut self, name: &str, index: usize) -> Compile<()> {
        self.exports.add(name, index)
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

    pub fn do_handler(&mut self) {
        self.stack.push(CompilerFrame::do_handler());
    }

    pub fn end_do_handler(&mut self) {
        let frame = self.stack.pop().expect("compiler frame underflow");
        match frame.scope {
            Scope::DoHandler => {}
            _ => panic!("expected do handler frame"),
        }
    }

    pub fn add_anon(&mut self) -> usize {
        self.top_mut().add_anon(BindingType::Let).index
    }
    pub fn add_let(&mut self, key: String) -> usize {
        self.top_mut().add(key, BindingType::Let).index
    }
    pub fn add_var(&mut self, key: String) -> usize {
        self.top_mut().add(key, BindingType::Var).index
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
        let res = self.get_inner(key, self.stack.len() - 1)?;
        Ok(vec![res.ir()])
    }

    fn get_inner(&mut self, key: &str, depth: usize) -> Compile<GetResult> {
        // check locals
        let frame = &mut self.stack[depth];
        if let Some(value) = frame.locals.get(key) {
            let res = match value.typ {
                BindingType::Let => GetResult::Local(value.index),
                BindingType::Var => GetResult::Var(value.index),
            };
            return Ok(res);
        }

        // check ivars
        if let Scope::Handler(instance) = &mut frame.scope {
            if let Some(res) = instance.get(key) {
                return Ok(res);
            }
        }

        // give up at root
        if let Scope::Root = frame.scope {
            return Err(CompileError::UnknownIdentifier(key.to_string()));
        }

        // recur at next level
        let result = self.get_inner(&key, depth - 1)?;

        // reject vars from outer scopes
        // second let frame needed for borrow reasons
        let frame = &mut self.stack[depth];
        if let Scope::Handler(_) = &frame.scope {
            if result.is_var() {
                return Err(CompileError::InvalidVarBinding);
            }
        }

        // transition result on way back up
        let out = match &mut frame.scope {
            Scope::Root => unreachable!(),
            Scope::Handler(instance) => instance.add(&key, result),
            Scope::DoHandler => result.parent(),
        };
        Ok(out)
    }

    pub fn set_var(&mut self, key: &str) -> CompileIR {
        let res = self.get_inner(&key, self.stack.len() - 1)?;
        return Ok(vec![res.assign_ir()]);
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
