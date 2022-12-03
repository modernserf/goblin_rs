use std::collections::HashMap;

use crate::{ir::IR, parse_stmt::Stmt, value::Value};

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
    fn offset_from(index: usize) -> Self {
        Self {
            index,
            map: HashMap::new(),
        }
    }
    fn allocated_since(&self, parent: &Locals) -> usize {
        self.index - parent.index
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
    fn allocate(&mut self, size: usize) {
        self.index += size;
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
            return Some(IR::IVar(record.index));
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
        return IR::IVar(index);
    }
    pub fn ivars(self) -> Vec<IR> {
        self.ivars
    }
}

#[derive(Debug)]
pub struct DoInstance {
    own_offset: usize,
    allocated: Vec<usize>,
}

impl DoInstance {
    fn new(own_offset: usize) -> Self {
        Self {
            own_offset,
            allocated: Vec::new(),
        }
    }
    fn push_allocated(&mut self, size: usize) {
        self.allocated.push(size)
    }
    fn max_allocated(&self) -> usize {
        self.allocated.iter().max().cloned().unwrap_or(0)
    }
}

#[derive(Debug)]
enum Scope {
    Root,
    Handler(Instance),
    DoHandler(DoInstance),
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
    fn do_handler(do_instance: DoInstance) -> Self {
        Self {
            locals: Locals::offset_from(do_instance.own_offset),
            scope: Scope::DoHandler(do_instance),
        }
    }
    fn add(&mut self, key: String, typ: BindingType) -> BindingRecord {
        self.locals.add(key, typ)
    }
}

#[derive(Debug)]
pub struct Compiler {
    stack: Vec<CompilerFrame>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            stack: vec![CompilerFrame::root()],
        }
    }
    pub fn program(program: Vec<Stmt>) -> CompileIR {
        let mut compiler = Self::new();
        Compiler::body(program, &mut compiler)
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

    pub fn do_instance(&self) -> DoInstance {
        DoInstance::new(self.top().locals.index)
    }
    pub fn do_handler(&mut self, do_instance: DoInstance) {
        let frame = CompilerFrame::do_handler(do_instance);
        self.stack.push(frame);
    }
    pub fn end_do_handler(&mut self) -> DoInstance {
        let frame = self.stack.pop().expect("compiler frame underflow");
        let mut do_instance = match frame.scope {
            Scope::DoHandler(x) => x,
            _ => panic!("expected do handler frame"),
        };
        let current = &self.top().locals;
        let allocated = frame.locals.allocated_since(current);
        do_instance.push_allocated(allocated);
        do_instance
    }
    pub fn end_do_instance(&mut self, do_instance: DoInstance) -> (usize, Vec<IR>) {
        let own_offset = do_instance.own_offset;
        let size = do_instance.max_allocated();
        self.top_mut().locals.allocate(size);
        (own_offset, vec![IR::Allocate(size)])
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
    // TODO: accept source, return Result<IR, CompileError>
    pub fn get(&mut self, key: &str) -> Option<IR> {
        for i in (0..self.stack.len()).rev() {
            let frame = &mut self.stack[i];
            if let Scope::Handler(instance) = &frame.scope {
                if let Some(ir) = instance.get(key) {
                    return Some(self.get_found(ir, key, i));
                }
            }
            if let Some(value) = frame.locals.get(key) {
                let ir = IR::Local(value.index);
                return Some(self.get_found(ir, key, i));
            }
        }
        None
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
    // TODO: accept source, return Result<usize, CompileError>
    pub fn get_var_index(&self, key: &str) -> Option<usize> {
        for i in (0..self.stack.len()).rev() {
            let frame = &self.stack[i];
            if let Some(value) = frame.locals.get(key) {
                match value.typ {
                    BindingType::Var => {
                        return Some(value.index);
                    }
                    _ => {
                        panic!("not a var")
                    }
                }
            }
            match frame.scope {
                Scope::DoHandler(_) => {}
                _ => return None,
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use crate::{
        class::{Class, Param},
        value::Value,
    };

    use super::*;

    fn compile(code: &str) -> CompileIR {
        let lexer = crate::lexer::Lexer::from_string(code);
        let mut parser = crate::parser::Parser::new(lexer);
        let program = parser.program().unwrap();
        Compiler::program(program)
    }

    fn assert_ok(code: &str, expected: Vec<IR>) {
        assert_eq!(compile(code), Ok(expected));
    }

    #[test]
    fn basics() {
        assert_ok("", vec![IR::Constant(Value::Unit)]);
        assert_ok("1", vec![IR::Constant(Value::Integer(1))]);
        assert_ok(
            "1 2",
            vec![
                IR::Constant(Value::Integer(1)),
                IR::Drop,
                IR::Constant(Value::Integer(2)),
            ],
        );
        assert_ok(
            "-1",
            vec![
                IR::Constant(Value::Integer(1)),
                IR::Send("-".to_string(), 0),
            ],
        );
        assert_ok(
            "1 + 2",
            vec![
                IR::Constant(Value::Integer(1)),
                IR::Constant(Value::Integer(2)),
                IR::Send("+:".to_string(), 1),
            ],
        );
        assert_ok(
            "let x := 1",
            vec![
                IR::Constant(Value::Integer(1)),
                IR::Assign(0),
                IR::Constant(Value::Unit),
            ],
        );
        assert_ok(
            "
            let x := 1
            x",
            vec![IR::Constant(Value::Integer(1)), IR::Assign(0), IR::Local(0)],
        );
        assert_ok(
            "[on {}]",
            vec![IR::Object(
                {
                    let mut class = Class::new();
                    class.add_handler("", vec![], vec![IR::Constant(Value::Unit)]);
                    class.rc()
                },
                0,
            )],
        );
        assert_ok(
            "[on {} 1]",
            vec![IR::Object(
                {
                    let mut class = Class::new();
                    class.add_handler("", vec![], vec![IR::Constant(Value::Integer(1))]);
                    class.rc()
                },
                0,
            )],
        );
    }

    #[test]
    fn params() {
        assert_ok(
            "[on {: x} x]",
            vec![IR::Object(
                {
                    let mut class = Class::new();
                    class.add_handler(":", vec![Param::Value], vec![IR::Local(0)]);
                    class.rc()
                },
                0,
            )],
        );
    }
}
