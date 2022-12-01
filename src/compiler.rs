use crate::{ir::IR, parse_stmt::Stmt, source::Source, value::Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Compiler {
    Root(Locals),
    Handler(Locals, Box<Instance>),
    DoHandler(Locals, Box<Compiler>),
}

impl Default for Compiler {
    fn default() -> Self {
        Self::root()
    }
}

impl Compiler {
    pub fn program(program: Vec<Stmt>) -> CompileResult {
        let mut compiler = Self::root();
        Compiler::body(&program, &mut compiler)
    }

    pub fn body(body: &[Stmt], compiler: &mut Compiler) -> CompileResult {
        if body.is_empty() {
            return Ok(vec![IR::Constant(Value::Unit)]);
        }

        let mut out = Vec::new();
        for i in 0..body.len() - 1 {
            let stmt = &body[i];
            let mut res = stmt.compile(compiler)?;
            out.append(&mut res);
        }
        match &body[body.len() - 1] {
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
        return Ok(out);
    }

    fn root() -> Self {
        Self::Root(Locals::root())
    }
    fn handler(instance: Instance) -> Self {
        Self::Handler(Locals::root(), Box::new(instance))
    }
    fn do_handler(self) -> Self {
        let index = match &self {
            Self::Root(ls) => ls.index,
            Self::Handler(ls, _) => ls.index,
            Self::DoHandler(ls, _) => ls.index,
        };
        Self::DoHandler(Locals::scope(index), Box::new(self))
    }
    fn take_instance(self) -> Instance {
        match self {
            Self::Handler(_, instance) => *instance,
            _ => unreachable!(),
        }
    }
    pub fn get(&mut self, key: &str) -> Option<IR> {
        match self {
            Self::Root(locals) => locals.get(key).map(|record| IR::Local(record.index)),
            Self::Handler(locals, instance) => {
                if let Some(record) = locals.get(key) {
                    return Some(IR::Local(record.index));
                }
                instance.get(key)
            }
            Self::DoHandler(locals, parent) => {
                if let Some(record) = locals.get(key) {
                    return Some(IR::Local(record.index));
                }
                parent.get(key)
            }
        }
    }
    fn get_outer(&mut self, key: &str) -> Option<IR> {
        // TODO: prevent referencing var / do
        self.get(key)
    }
    pub fn add_let(&mut self, key: String) -> ScopeRecord {
        self.add(key, ScopeType::Let)
    }
    fn add(&mut self, key: String, typ: ScopeType) -> ScopeRecord {
        self.locals().add(key, typ)
    }
    pub fn with_instance(
        &mut self,
        mut f: impl FnMut(&mut Instance) -> CompileOk,
    ) -> CompileResult {
        let parent = std::mem::take(self);
        let mut instance = Instance::new(parent);
        f(&mut instance)?;
        *self = instance.parent;
        Ok(instance.ivars)
    }

    pub fn with_do_block(
        &mut self,
        mut f: impl FnMut(&mut Compiler) -> CompileOk,
    ) -> Result<usize, CompileError> {
        let parent = std::mem::take(self);
        let mut compiler = parent.do_handler();
        f(&mut compiler)?;
        let end_index;
        *self = match compiler {
            Self::DoHandler(locals, parent) => {
                end_index = locals.index;
                *parent
            }
            _ => unreachable!(),
        };
        let locals = self.locals();
        let start_index = locals.index;
        let allocated = end_index - start_index;
        Ok(allocated)
    }

    fn locals(&mut self) -> &mut Locals {
        match self {
            Self::DoHandler(ls, _) => ls,
            Self::Handler(ls, _) => ls,
            Self::Root(ls) => ls,
        }
    }

    pub fn allocate(&mut self, size: usize) -> CompileResult {
        self.locals().allocate(size);
        Ok(vec![IR::Allocate(size)])
    }

    pub fn push_self(&self, source: Source) -> CompileResult {
        match self {
            Self::Root(_) => Err(CompileError::InvalidSelf(source)),
            Self::Handler(_, _) => Ok(vec![IR::SelfRef]),
            Self::DoHandler(_, _) => Ok(vec![IR::SelfRef]),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompileError {
    UnknownIdentifier(String, Source),
    InvalidSelf(Source),
}

pub type CompileResult = Result<Vec<IR>, CompileError>;
pub type CompileOk = Result<(), CompileError>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ScopeType {
    Let,
}
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ScopeRecord {
    pub index: usize,
    pub typ: ScopeType,
}

#[derive(Debug, Clone, Default)]
pub struct Instance {
    parent: Compiler,
    ivars: Vec<IR>,
    ivar_map: HashMap<String, ScopeRecord>,
}

impl Instance {
    fn new(parent: Compiler) -> Self {
        Self {
            parent,
            ivars: Vec::new(),
            ivar_map: HashMap::new(),
        }
    }
    pub fn with_handler(&mut self, mut f: impl FnMut(&mut Compiler) -> CompileOk) -> CompileOk {
        let instance = std::mem::take(self);
        let mut handler = Compiler::handler(instance);
        f(&mut handler)?;
        *self = handler.take_instance();
        Ok(())
    }
    fn get(&mut self, key: &str) -> Option<IR> {
        if let Some(record) = self.ivar_map.get(key) {
            return Some(IR::IVar(record.index));
        }
        let index = self.ivars.len();
        if let Some(value) = self.parent.get_outer(key) {
            self.ivars.push(value);
            self.ivar_map.insert(
                key.to_string(),
                ScopeRecord {
                    index,
                    typ: ScopeType::Let,
                },
            );
            return Some(IR::IVar(index));
        }
        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Locals {
    index: usize,
    map: HashMap<String, ScopeRecord>,
}

impl Locals {
    fn root() -> Self {
        Self {
            index: 0,
            map: HashMap::new(),
        }
    }
    fn scope(index: usize) -> Self {
        Self {
            index,
            map: HashMap::new(),
        }
    }
    fn get(&self, key: &str) -> Option<ScopeRecord> {
        self.map.get(key).map(|r| r.to_owned())
    }
    fn add(&mut self, key: String, typ: ScopeType) -> ScopeRecord {
        let record = ScopeRecord {
            index: self.index,
            typ,
        };
        self.index += 1;
        self.map.insert(key, record);
        record
    }
    fn allocate(&mut self, size: usize) {
        self.index += size;
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    fn compile(code: &str) -> CompileResult {
        let lexer = Lexer::from_string(code);
        let mut parser = Parser::new(lexer);
        let program = parser.program().unwrap();
        Compiler::program(program)
    }

    #[test]
    fn numbers() {
        assert!(compile("0").is_ok());
        assert!(compile("123_45").is_ok());
    }

    #[test]
    fn objects() {
        assert!(compile(
            "let x := [
                    on {} 1
                    on {foo} 2
                    on {bar: arg} arg
                ]
                x{foo}
                "
        )
        .is_ok())
    }
}
