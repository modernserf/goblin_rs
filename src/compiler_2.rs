use crate::{
    parser_2::Parse,
    runtime_2::{Address, Class, Index, Param, Selector, IR},
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    UnknownIdentifier(String),
    InvalidSet(String),
    InvalidVarReference(String),
    InvalidVarArg(String),
    InvalidDoReference(String),
}
pub type Compile<T> = Result<T, CompileError>;
pub type CompileIR = Compile<IRBuilder>;

pub struct IRBuilder {
    ir: Vec<IR>,
}
impl IRBuilder {
    fn new() -> Self {
        IRBuilder { ir: Vec::new() }
    }
    fn from(ir: Vec<IR>) -> Self {
        IRBuilder { ir }
    }
    fn push(&mut self, item: IR) {
        self.ir.push(item);
    }
    fn append(&mut self, other: IRBuilder) {
        let mut other_ir = other.to_vec();
        self.ir.append(&mut other_ir);
    }
    fn to_vec(self) -> Vec<IR> {
        self.ir
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    Identifier(String),
    VarIdentifier(String),
    DoIdentifier(String),
}
impl Binding {
    fn compile_let(self, compiler: &mut Compiler) {
        match self {
            Self::Identifier(name) => compiler.add_let(name),
            _ => todo!(),
        }
    }
    fn compile_var(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Identifier(name) => compiler.add_var(name),
            _ => todo!(),
        }
    }
    fn compile_set(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Identifier(name) => compiler.set(name),
            _ => todo!(),
        }
    }
    fn compile_param(self, compiler: &mut Compiler) -> Param {
        match self {
            Self::Identifier(name) => {
                compiler.add_let(name);
                Param::Value
            }
            Self::VarIdentifier(name) => {
                compiler.add_var_param(name);
                Param::Var
            }
            Self::DoIdentifier(name) => {
                compiler.add_do_param(name);
                Param::Do
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Let(Binding, Expr),
    Var(Binding, Expr),
    Set(Binding, Expr),
}

impl Stmt {
    fn compile_base(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Expr(expr) => expr.compile(compiler),
            Self::Let(binding, expr) => {
                let ir = expr.compile(compiler)?;
                binding.compile_let(compiler);
                Ok(ir)
            }
            Self::Var(binding, expr) => {
                let mut ir = expr.compile(compiler)?;
                let var_ir = binding.compile_var(compiler)?;
                ir.append(var_ir);
                Ok(ir)
            }
            Self::Set(binding, expr) => {
                let mut ir = expr.compile(compiler)?;
                ir.append(binding.compile_set(compiler)?);
                Ok(ir)
            }
        }
    }
    // remove unused stack values
    fn compile_most(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Expr(_) => {
                let mut ir = self.compile_base(compiler)?;
                ir.push(IR::Drop);
                Ok(ir)
            }
            _ => self.compile_base(compiler),
        }
    }
    // add stack value if not present
    fn compile_last(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Expr(_) => self.compile_base(compiler),
            _ => {
                let mut ir = self.compile_base(compiler)?;
                ir.push(IR::Unit);
                Ok(ir)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Identifier(String),
    Send(Selector, Box<Expr>, Vec<Expr>),
    Object(Object),
    VarArg(String),
    DoArg(Object),
}

impl Expr {
    fn compile(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Integer(value) => Ok(IRBuilder::from(vec![IR::Integer(value)])),
            Self::Identifier(name) => compiler.identifier(name),
            Self::Send(selector, target, args) => {
                let mut ir = IRBuilder::new();
                for arg in args {
                    ir.append(arg.compile_arg(compiler)?);
                }
                ir.append(target.compile_target(compiler)?);
                ir.push(IR::Send(selector));
                Ok(ir)
            }
            Self::Object(obj) => obj.compile(compiler),
            Self::VarArg(_) => unreachable!(),
            Self::DoArg(_) => unreachable!(),
        }
    }
    fn compile_arg(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::VarArg(name) => compiler.var_arg(name),
            Self::DoArg(obj) => obj.compile_do(compiler),
            Self::Identifier(name) => compiler.arg_identifier(name),
            _ => self.compile(compiler),
        }
    }
    fn compile_target(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::DoArg(obj) => obj.compile_do(compiler),
            Self::Identifier(name) => compiler.target_identifier(name),
            _ => self.compile(compiler),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Handler {
    params: Vec<Binding>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    handlers: HashMap<String, Handler>,
}
impl Object {
    pub fn new() -> Self {
        Object {
            handlers: HashMap::new(),
        }
    }
    #[cfg(test)]
    fn add(&mut self, selector: &str, params: Vec<Binding>, body: Vec<Stmt>) {
        self.add_handler(selector.to_string(), params, body)
            .unwrap()
    }
    pub fn add_handler(
        &mut self,
        selector: String,
        params: Vec<Binding>,
        body: Vec<Stmt>,
    ) -> Parse<()> {
        if self
            .handlers
            .insert(selector, Handler { params, body })
            .is_some()
        {
            todo!()
        }
        Ok(())
    }
    fn compile(self, compiler: &mut Compiler) -> CompileIR {
        let mut class = Class::new();
        let mut ivals = IVals::new();

        for (selector, handler) in self.handlers {
            compiler.handler(ivals);
            let mut params = vec![];
            for param in handler.params {
                params.push(param.compile_param(compiler));
            }
            let ir = compiler.body(handler.body)?;

            class.add_handler(selector, params, ir.to_vec());
            ivals = compiler.end_handler();
        }
        class.set_ivals(ivals.count());
        let mut out = ivals.compile()?;
        out.push(IR::Object(class.rc()));
        Ok(out)
    }
    fn compile_do(self, compiler: &mut Compiler) -> CompileIR {
        let mut class = Class::new();
        let mut ivals = IVals::new();
        for (selector, handler) in self.handlers {
            compiler.do_handler(ivals);
            let mut params = vec![];
            for param in handler.params {
                params.push(param.compile_param(compiler));
            }
            let ir = compiler.body(handler.body)?;

            class.add_handler(selector, params, ir.to_vec());
            ivals = compiler.end_handler();
        }
        class.set_ivals(ivals.count());
        let mut out = ivals.compile()?;
        out.push(IR::DoObject(class.rc()));
        Ok(out)
    }
}

#[derive(Debug, Clone, Copy)]
enum BindingRecord {
    Local(Address),
    Var(Address),
    Do(Address),
    IVal(Index),
    VarIVal(Index),
    DoIVal(Index),
}

impl BindingRecord {
    fn ival(self) -> IR {
        match self {
            Self::Local(addr) => IR::Local(addr),
            Self::Var(addr) => IR::Local(addr),
            Self::Do(addr) => IR::Local(addr),
            Self::IVal(index) => IR::IVal(index),
            Self::VarIVal(index) => IR::IVal(index),
            Self::DoIVal(index) => IR::IVal(index),
        }
    }
    fn identifier(self, key: String) -> CompileIR {
        let ir = match self {
            Self::Local(address) => vec![IR::Local(address)],
            Self::Var(address) => vec![IR::Local(address), IR::Deref],
            Self::IVal(index) => vec![IR::IVal(index)],
            Self::VarIVal(index) => vec![IR::IVal(index), IR::Deref],
            Self::Do(_) => return Err(CompileError::InvalidDoReference(key)),
            Self::DoIVal(_) => return Err(CompileError::InvalidDoReference(key)),
        };
        Ok(IRBuilder::from(ir))
    }
    fn target_identifier(self, key: String) -> CompileIR {
        let ir = match self {
            Self::Do(address) => vec![IR::Local(address)],
            Self::DoIVal(index) => vec![IR::IVal(index)],
            _ => return self.identifier(key),
        };
        Ok(IRBuilder::from(ir))
    }
    fn arg_identifier(self, key: String) -> CompileIR {
        let ir = match self {
            Self::Do(address) => vec![IR::Local(address)],
            Self::DoIVal(index) => vec![IR::IVal(index)],
            _ => return self.identifier(key),
        };
        Ok(IRBuilder::from(ir))
    }
    fn var_arg(self, key: String) -> CompileIR {
        let ir = match self {
            Self::Var(address) => IR::Local(address),
            _ => return Err(CompileError::InvalidVarArg(key)),
        };
        Ok(IRBuilder::from(vec![ir]))
    }
    fn set(self, key: String) -> CompileIR {
        let ir = match self {
            Self::Var(address) => vec![IR::Local(address), IR::SetVar],
            Self::VarIVal(index) => vec![IR::IVal(index), IR::SetVar],
            _ => return Err(CompileError::InvalidSet(key)),
        };
        Ok(IRBuilder::from(ir))
    }
    fn as_handler_ival(self, next_index: Index, key: &str) -> Compile<Self> {
        match self {
            Self::Local(_) => Ok(Self::IVal(next_index)),
            Self::IVal(_) => Ok(Self::IVal(next_index)),
            _ => Err(CompileError::InvalidVarReference(key.to_string())),
        }
    }
    fn as_do_handler_ival(self, next_index: Index) -> Self {
        match self {
            Self::Local(_) => Self::IVal(next_index),
            Self::IVal(_) => Self::IVal(next_index),
            Self::Var(_) => Self::VarIVal(next_index),
            Self::VarIVal(_) => Self::VarIVal(next_index),
            Self::Do(_) => Self::DoIVal(next_index),
            Self::DoIVal(_) => Self::DoIVal(next_index),
        }
    }
}

struct Locals {
    locals: HashMap<String, BindingRecord>,
    next_index: usize,
}

impl Locals {
    fn new() -> Self {
        Locals {
            locals: HashMap::new(),
            next_index: 0,
        }
    }
    fn get(&self, key: &str) -> Option<BindingRecord> {
        self.locals.get(key).map(|x| *x)
    }
    fn add_let(&mut self, key: String) {
        self.locals
            .insert(key, BindingRecord::Local(self.next_index));
        self.next_index += 1;
    }
    fn add_var(&mut self, key: String) -> usize {
        let var_index = self.next_index;
        self.locals.insert(key, BindingRecord::Var(var_index + 1));
        // allocate for both var ptr & value it references
        self.next_index += 2;
        var_index
    }
    fn add_var_param(&mut self, key: String) {
        self.locals.insert(key, BindingRecord::Var(self.next_index));
        self.next_index += 1;
    }
    fn add_do_param(&mut self, key: String) {
        self.locals.insert(key, BindingRecord::Do(self.next_index));
        self.next_index += 1;
    }
}

struct IVals {
    map: HashMap<String, BindingRecord>,
    ivals: Vec<BindingRecord>,
}
impl IVals {
    fn new() -> Self {
        IVals {
            map: HashMap::new(),
            ivals: Vec::new(),
        }
    }
    fn add(&mut self, key: String, value: BindingRecord) -> Compile<BindingRecord> {
        let next_index = self.ivals.len();
        self.ivals.push(value);
        let ival = value.as_handler_ival(next_index, &key)?;
        if self.map.insert(key, ival.clone()).is_some() {
            panic!("duplicate ival key")
        }
        Ok(ival)
    }
    fn add_do(&mut self, key: String, value: BindingRecord) -> Compile<BindingRecord> {
        let next_index = self.ivals.len();
        self.ivals.push(value);
        let ival = value.as_do_handler_ival(next_index);
        if self.map.insert(key, ival.clone()).is_some() {
            panic!("duplicate ival key")
        }
        Ok(ival)
    }
    fn get(&self, key: &str) -> Option<BindingRecord> {
        self.map.get(key).cloned()
    }
    fn count(&self) -> usize {
        self.ivals.len()
    }
    fn compile(self) -> CompileIR {
        let mut out = IRBuilder::new();
        for ival in self.ivals {
            out.push(ival.ival());
        }
        Ok(out)
    }
}
enum CompilerFrame {
    Root(Locals),
    Handler(Locals, IVals),
    Do(Locals, IVals),
}

impl CompilerFrame {
    fn root() -> Self {
        Self::Root(Locals::new())
    }
    fn handler(ivals: IVals) -> Self {
        Self::Handler(Locals::new(), ivals)
    }
    fn do_handler(ivals: IVals) -> Self {
        Self::Do(Locals::new(), ivals)
    }
    fn get_local(&self, key: &str) -> Option<BindingRecord> {
        match self {
            Self::Root(ls) => ls.get(key),
            Self::Handler(ls, _) => ls.get(key),
            Self::Do(ls, _) => ls.get(key),
        }
    }
    fn locals_mut(&mut self) -> &mut Locals {
        match self {
            Self::Root(ls) => ls,
            Self::Handler(ls, _) => ls,
            Self::Do(ls, _) => ls,
        }
    }
    fn ivals(self) -> IVals {
        match self {
            Self::Root(_) => panic!("no ivals at root"),
            Self::Handler(_, ivals) => ivals,
            Self::Do(_, ivals) => ivals,
        }
    }
    fn get_ival(&self, key: &str) -> Option<BindingRecord> {
        match self {
            Self::Root(_) => None,
            Self::Handler(_, ivals) => ivals.get(key),
            Self::Do(_, ivals) => ivals.get(key),
        }
    }
    fn add_ival(&mut self, key: String, value: BindingRecord) -> Compile<BindingRecord> {
        match self {
            Self::Root(_) => panic!("no ivals at root"),
            Self::Handler(_, ivals) => ivals.add(key, value),
            Self::Do(_, ivals) => ivals.add_do(key, value),
        }
    }
}

pub struct Compiler {
    frames: Vec<CompilerFrame>,
}

impl Compiler {
    pub fn program(program: Vec<Stmt>) -> Compile<Vec<IR>> {
        let mut compiler = Compiler::new();
        let out = compiler.body(program)?;
        Ok(out.to_vec())
    }
    fn new() -> Self {
        Compiler {
            frames: vec![CompilerFrame::root()],
        }
    }
    fn body(&mut self, mut body: Vec<Stmt>) -> CompileIR {
        let mut builder = IRBuilder::new();
        if body.len() == 0 {
            builder.push(IR::Unit);
            return Ok(builder);
        }

        let last = body.pop().unwrap();
        for stmt in body {
            builder.append(stmt.compile_most(self)?);
        }
        builder.append(last.compile_last(self)?);

        Ok(builder)
    }
    fn handler(&mut self, ivals: IVals) {
        self.frames.push(CompilerFrame::handler(ivals))
    }
    fn do_handler(&mut self, ivals: IVals) {
        self.frames.push(CompilerFrame::do_handler(ivals))
    }
    fn end_handler(&mut self) -> IVals {
        self.frames.pop().unwrap().ivals()
    }
    fn top_mut(&mut self) -> &mut CompilerFrame {
        self.frames.last_mut().unwrap()
    }
    fn add_let(&mut self, key: String) {
        self.top_mut().locals_mut().add_let(key);
    }
    fn add_var(&mut self, key: String) -> CompileIR {
        let index = self.top_mut().locals_mut().add_var(key);
        Ok(IRBuilder::from(vec![IR::Var(index)]))
    }
    fn add_var_param(&mut self, key: String) {
        self.top_mut().locals_mut().add_var_param(key);
    }
    fn add_do_param(&mut self, key: String) {
        self.top_mut().locals_mut().add_do_param(key);
    }
    fn identifier(&mut self, key: String) -> CompileIR {
        self.get(&key)?.identifier(key)
    }
    fn target_identifier(&mut self, key: String) -> CompileIR {
        self.get(&key)?.target_identifier(key)
    }
    fn arg_identifier(&mut self, key: String) -> CompileIR {
        self.get(&key)?.arg_identifier(key)
    }
    fn var_arg(&mut self, key: String) -> CompileIR {
        self.get(&key)?.var_arg(key)
    }
    fn set(&mut self, key: String) -> CompileIR {
        self.get(&key)?.set(key)
    }
    fn get(&mut self, key: &str) -> Compile<BindingRecord> {
        self.get_at_depth(key, self.frames.len() - 1)
    }
    fn get_at_depth(&mut self, key: &str, depth: usize) -> Compile<BindingRecord> {
        let frame = &self.frames[depth];
        if let Some(record) = frame.get_local(key) {
            return Ok(record);
        }
        if let Some(record) = frame.get_ival(key) {
            return Ok(record);
        }
        if depth == 0 {
            return Err(CompileError::UnknownIdentifier(key.to_string()));
        }
        let next = self.get_at_depth(key, depth - 1)?;

        let frame = &mut self.frames[depth];
        let ival = frame.add_ival(key.to_string(), next)?;
        Ok(ival)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_ok(code: Vec<Stmt>, expected: Vec<IR>) {
        assert_eq!(Compiler::program(code), Ok(expected))
    }

    fn assert_err(code: Vec<Stmt>, expected: CompileError) {
        assert_eq!(Compiler::program(code), Err(expected))
    }

    fn int(val: i64) -> Expr {
        Expr::Integer(val)
    }
    fn b_ident(name: &str) -> Binding {
        Binding::Identifier(name.to_string())
    }
    fn b_var(name: &str) -> Binding {
        Binding::VarIdentifier(name.to_string())
    }
    fn b_do(name: &str) -> Binding {
        Binding::DoIdentifier(name.to_string())
    }
    fn ident(name: &str) -> Expr {
        Expr::Identifier(name.to_string())
    }
    fn var_arg(name: &str) -> Expr {
        Expr::VarArg(name.to_string())
    }

    fn send(target: Expr, selector: &str, args: Vec<Expr>) -> Expr {
        Expr::Send(selector.to_string(), Box::new(target), args)
    }

    #[test]
    fn empty() {
        assert_ok(vec![], vec![IR::Unit]);
    }

    #[test]
    fn integer() {
        assert_ok(vec![Stmt::Expr(int(123))], vec![IR::Integer(123)])
    }

    #[test]
    fn identifier() {
        assert_ok(
            vec![
                Stmt::Let(b_ident("foo"), int(123)),
                Stmt::Let(b_ident("bar"), int(456)),
                Stmt::Expr(ident("foo")),
            ],
            vec![IR::Integer(123), IR::Integer(456), IR::Local(0)],
        )
    }

    #[test]
    fn unknown_identifier() {
        assert_err(
            vec![Stmt::Expr(ident("foo"))],
            CompileError::UnknownIdentifier("foo".to_string()),
        )
    }

    #[test]
    fn send_values() {
        assert_ok(
            vec![Stmt::Expr(send(int(123), "+:", vec![int(456)]))],
            vec![
                IR::Integer(456),
                IR::Integer(123),
                IR::Send("+:".to_string()),
            ],
        );
    }

    #[test]
    fn vars() {
        assert_ok(
            vec![
                Stmt::Let(b_ident("foo"), int(456)),
                Stmt::Var(b_ident("bar"), int(123)),
                Stmt::Set(b_ident("bar"), int(789)),
                Stmt::Expr(ident("bar")),
            ],
            vec![
                IR::Integer(456),
                IR::Integer(123),
                IR::Var(1),
                IR::Integer(789),
                IR::Local(2),
                IR::SetVar,
                IR::Local(2),
                IR::Deref,
            ],
        )
    }

    #[test]
    fn unknown_var() {
        assert_err(
            vec![Stmt::Set(b_ident("bar"), int(789))],
            CompileError::UnknownIdentifier("bar".to_string()),
        )
    }

    #[test]
    fn invalid_set() {
        assert_err(
            vec![
                Stmt::Let(b_ident("foo"), int(456)),
                Stmt::Set(b_ident("foo"), int(789)),
            ],
            CompileError::InvalidSet("foo".to_string()),
        )
    }

    #[test]
    fn empty_object() {
        assert_ok(
            vec![Stmt::Expr(Expr::Object(Object::new()))],
            vec![IR::Object(Class::new().rc())],
        )
    }

    #[test]
    fn object_with_simple_handler() {
        assert_ok(
            vec![
                Stmt::Let(b_ident("foo"), int(123)),
                Stmt::Let(b_ident("bar"), int(456)),
                Stmt::Expr(Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "handler",
                        vec![],
                        vec![
                            Stmt::Let(b_ident("bar"), int(789)),
                            Stmt::Expr(ident("bar")),
                        ],
                    );
                    obj
                })),
            ],
            vec![
                IR::Integer(123),
                IR::Integer(456),
                IR::Object({
                    let mut class = Class::new();
                    class.add("handler", vec![], vec![IR::Integer(789), IR::Local(0)]);

                    class.rc()
                }),
            ],
        )
    }

    #[test]
    fn object_with_args() {
        assert_ok(
            vec![
                Stmt::Let(b_ident("foo"), int(123)),
                Stmt::Let(b_ident("bar"), int(456)),
                Stmt::Expr(Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "handler:",
                        vec![b_ident("foo")],
                        vec![
                            Stmt::Let(b_ident("bar"), int(789)),
                            Stmt::Expr(send(ident("foo"), "+:", vec![ident("bar")])),
                        ],
                    );
                    obj
                })),
            ],
            vec![
                IR::Integer(123),
                IR::Integer(456),
                IR::Object({
                    let mut class = Class::new();
                    class.add(
                        "handler:",
                        vec![Param::Value],
                        vec![
                            IR::Integer(789),
                            IR::Local(1),
                            IR::Local(0),
                            IR::Send("+:".to_string()),
                        ],
                    );

                    class.rc()
                }),
            ],
        )
    }

    #[test]
    fn instance_values() {
        assert_ok(
            vec![
                Stmt::Let(b_ident("foo"), int(123)),
                Stmt::Expr(Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "handler:",
                        vec![b_ident("bar")],
                        vec![Stmt::Expr(send(ident("foo"), "+:", vec![ident("bar")]))],
                    );
                    obj
                })),
            ],
            vec![
                IR::Integer(123),
                IR::Local(0),
                IR::Object({
                    let mut class = Class::new();
                    class.add(
                        "handler:",
                        vec![Param::Value],
                        vec![IR::Local(0), IR::IVal(0), IR::Send("+:".to_string())],
                    );
                    class.set_ivals(1);

                    class.rc()
                }),
            ],
        )
    }

    #[test]
    fn instance_values_var_error() {
        assert_err(
            vec![
                Stmt::Var(b_ident("foo"), int(123)),
                Stmt::Expr(Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "handler:",
                        vec![b_ident("bar")],
                        vec![Stmt::Expr(send(ident("foo"), "+:", vec![ident("bar")]))],
                    );
                    obj
                })),
            ],
            CompileError::InvalidVarReference("foo".to_string()),
        )
    }

    #[test]
    fn instance_values_shared() {
        assert_ok(
            vec![
                Stmt::Let(b_ident("foo"), int(123)),
                Stmt::Expr(Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "handler:",
                        vec![b_ident("bar")],
                        vec![Stmt::Expr(send(ident("foo"), "+:", vec![ident("bar")]))],
                    );
                    obj.add("other", vec![], vec![Stmt::Expr(ident("foo"))]);
                    obj
                })),
            ],
            vec![
                IR::Integer(123),
                IR::Local(0),
                IR::Object({
                    let mut class = Class::new();
                    class.add(
                        "handler:",
                        vec![Param::Value],
                        vec![IR::Local(0), IR::IVal(0), IR::Send("+:".to_string())],
                    );
                    class.add("other", vec![], vec![IR::IVal(0)]);
                    class.set_ivals(1);

                    class.rc()
                }),
            ],
        )
    }

    #[test]
    fn var_args() {
        assert_ok(
            vec![
                Stmt::Let(
                    b_ident("obj"),
                    Expr::Object({
                        let mut obj = Object::new();
                        obj.add(
                            "handler:",
                            vec![b_var("arg")],
                            vec![Stmt::Set(b_ident("arg"), int(10))],
                        );
                        obj
                    }),
                ),
                Stmt::Var(b_ident("foo"), int(5)),
                Stmt::Expr(send(ident("obj"), "handler:", vec![var_arg("foo")])),
                Stmt::Expr(ident("foo")),
            ],
            vec![
                IR::Object({
                    let mut class = Class::new();
                    class.add(
                        "handler:",
                        vec![Param::Var],
                        vec![IR::Integer(10), IR::Local(0), IR::SetVar, IR::Unit],
                    );
                    class.rc()
                }),
                IR::Integer(5),
                IR::Var(1),
                IR::Local(2),
                IR::Local(0),
                IR::Send("handler:".to_string()),
                IR::Drop,
                IR::Local(2),
                IR::Deref,
            ],
        )
    }

    #[test]
    fn invalid_var_arg() {
        assert_err(
            vec![
                Stmt::Let(
                    b_ident("obj"),
                    Expr::Object({
                        let mut obj = Object::new();
                        obj.add(
                            "handler:",
                            vec![b_var("arg")],
                            vec![Stmt::Set(b_ident("arg"), int(10)), Stmt::Expr(ident("arg"))],
                        );
                        obj
                    }),
                ),
                Stmt::Let(b_ident("foo"), int(5)),
                Stmt::Expr(send(ident("obj"), "handler:", vec![var_arg("foo")])),
                Stmt::Expr(ident("foo")),
            ],
            CompileError::InvalidVarArg("foo".to_string()),
        );
    }

    #[test]
    fn do_arg() {
        assert_ok(
            vec![Stmt::Expr(send(
                Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        ":",
                        vec![b_do("f")],
                        vec![Stmt::Expr(send(ident("f"), "foo", vec![]))],
                    );
                    obj
                }),
                ":",
                vec![Expr::DoArg({
                    let mut obj = Object::new();
                    obj.add("foo", vec![], vec![Stmt::Expr(int(123))]);

                    obj
                })],
            ))],
            vec![
                IR::DoObject({
                    let mut class = Class::new();
                    class.add("foo", vec![], vec![IR::Integer(123)]);

                    class.rc()
                }),
                IR::Object({
                    let mut class = Class::new();
                    class.add(
                        ":",
                        vec![Param::Do],
                        vec![IR::Local(0), IR::Send("foo".to_string())],
                    );

                    class.rc()
                }),
                IR::Send(":".to_string()),
            ],
        )
    }

    #[test]
    fn do_arg_var_closure() {
        assert_ok(
            vec![
                Stmt::Var(b_ident("state"), int(123)),
                Stmt::Expr(send(
                    Expr::Object({
                        let mut obj = Object::new();
                        obj.add(
                            ":",
                            vec![b_do("f")],
                            vec![Stmt::Expr(send(ident("f"), "foo", vec![]))],
                        );
                        obj
                    }),
                    ":",
                    vec![Expr::DoArg({
                        let mut obj = Object::new();
                        obj.add("foo", vec![], vec![Stmt::Set(b_ident("state"), int(456))]);
                        obj
                    })],
                )),
            ],
            vec![
                IR::Integer(123),
                IR::Var(0),
                IR::Local(1),
                IR::DoObject({
                    let mut class = Class::new();
                    class.add(
                        "foo",
                        vec![],
                        vec![IR::Integer(456), IR::IVal(0), IR::SetVar, IR::Unit],
                    );
                    class.set_ivals(1);

                    class.rc()
                }),
                IR::Object({
                    let mut class = Class::new();
                    class.add(
                        ":",
                        vec![Param::Do],
                        vec![IR::Local(0), IR::Send("foo".to_string())],
                    );

                    class.rc()
                }),
                IR::Send(":".to_string()),
            ],
        )
    }

    #[test]
    fn invalid_do_arg_reference() {
        assert_err(
            vec![Stmt::Expr(send(
                Expr::Object({
                    let mut object = Object::new();
                    object.add(
                        ":",
                        vec![b_do("f")],
                        vec![Stmt::Let(b_ident("g"), ident("f"))],
                    );
                    object
                }),
                ":",
                vec![Expr::DoArg(Object::new())],
            ))],
            CompileError::InvalidDoReference("f".to_string()),
        )
    }

    #[test]
    fn do_arg_closure() {
        assert_ok(
            vec![Stmt::Expr(Expr::Object({
                let mut obj = Object::new();
                obj.add(
                    ":",
                    vec![b_do("f")],
                    vec![Stmt::Expr(send(
                        Expr::Object(Object::new()),
                        ":",
                        vec![Expr::DoArg({
                            let mut obj = Object::new();
                            obj.add(
                                "foo",
                                vec![],
                                vec![Stmt::Expr(send(ident("f"), "foo", vec![]))],
                            );
                            obj
                        })],
                    ))],
                );
                obj
            }))],
            vec![IR::Object({
                let mut class = Class::new();
                class.add(
                    ":",
                    vec![Param::Do],
                    vec![
                        IR::Local(0),
                        IR::DoObject({
                            let mut class = Class::new();
                            class.add(
                                "foo",
                                vec![],
                                vec![IR::IVal(0), IR::Send("foo".to_string())],
                            );
                            class.set_ivals(1);
                            class.rc()
                        }),
                        IR::Object(Class::new().rc()),
                        IR::Send(":".to_string()),
                    ],
                );
                class.rc()
            })],
        );
    }
}
