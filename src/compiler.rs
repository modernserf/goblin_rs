use crate::{
    ast::Stmt,
    ir::{Address, Class, Index, IR},
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    UnknownIdentifier(String),
    InvalidSet(String),
    InvalidVarReference(String),
    InvalidVarArg(String),
    InvalidDoReference(String),
    DuplicateExport(String),
    InvalidExport(String),
}

pub type Compile<T> = Result<T, CompileError>;
pub type CompileIR = Compile<IRBuilder>;

pub struct IRBuilder {
    ir: Vec<IR>,
}
impl IRBuilder {
    pub fn new() -> Self {
        IRBuilder { ir: Vec::new() }
    }
    pub fn from(ir: Vec<IR>) -> Self {
        IRBuilder { ir }
    }
    pub fn push(&mut self, item: IR) {
        self.ir.push(item);
    }
    pub fn append(&mut self, other: IRBuilder) {
        let mut other_ir = other.to_vec();
        self.ir.append(&mut other_ir);
    }
    pub fn to_vec(self) -> Vec<IR> {
        self.ir
    }
}

struct Exports {
    exports: HashMap<String, Address>,
}

impl Exports {
    fn new() -> Self {
        Exports {
            exports: HashMap::new(),
        }
    }
    fn add(&mut self, name: String, address: Address) -> Compile<()> {
        if self.exports.contains_key(&name) {
            return Err(CompileError::DuplicateExport(name));
        }
        self.exports.insert(name, address);
        Ok(())
    }
    fn compile(self) -> CompileIR {
        let mut ir = IRBuilder::new();
        let mut class = Class::new();
        let arity = self.exports.len();
        for (i, (key, addr)) in self.exports.into_iter().enumerate() {
            ir.push(IR::Local(addr));
            class.add_handler(key, vec![], vec![IR::IVal(i)]);
        }
        ir.push(IR::Object(class.rc(), arity));
        Ok(ir)
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
            Self::VarIVal(index) => IR::IVal(index),
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
    fn add_anon(&mut self) -> usize {
        let address = self.next_index;
        self.next_index += 1;
        address
    }
    fn add_let(&mut self, key: String) -> usize {
        let address = self.next_index;
        self.locals
            .insert(key, BindingRecord::Local(self.next_index));
        self.next_index += 1;
        address
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

pub struct IVals {
    map: HashMap<String, BindingRecord>,
    ivals: Vec<BindingRecord>,
}
impl IVals {
    pub fn new() -> Self {
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
    pub fn count(&self) -> usize {
        self.ivals.len()
    }
    pub fn compile(self) -> CompileIR {
        let mut out = IRBuilder::new();
        for ival in self.ivals {
            out.push(ival.ival());
        }
        Ok(out)
    }
}
enum CompilerFrame {
    Root(Locals, Exports),
    Handler(Locals, IVals),
    Do(Locals, IVals),
}

impl CompilerFrame {
    fn root() -> Self {
        Self::Root(Locals::new(), Exports::new())
    }
    fn handler(ivals: IVals) -> Self {
        Self::Handler(Locals::new(), ivals)
    }
    fn do_handler(ivals: IVals) -> Self {
        Self::Do(Locals::new(), ivals)
    }
    fn get_local(&self, key: &str) -> Option<BindingRecord> {
        match self {
            Self::Root(ls, _) => ls.get(key),
            Self::Handler(ls, _) => ls.get(key),
            Self::Do(ls, _) => ls.get(key),
        }
    }
    fn locals_mut(&mut self) -> &mut Locals {
        match self {
            Self::Root(ls, _) => ls,
            Self::Handler(ls, _) => ls,
            Self::Do(ls, _) => ls,
        }
    }
    fn ivals(self) -> IVals {
        match self {
            Self::Root(_, _) => panic!("no ivals at root"),
            Self::Handler(_, ivals) => ivals,
            Self::Do(_, ivals) => ivals,
        }
    }
    fn get_ival(&self, key: &str) -> Option<BindingRecord> {
        match self {
            Self::Root(_, _) => None,
            Self::Handler(_, ivals) => ivals.get(key),
            Self::Do(_, ivals) => ivals.get(key),
        }
    }
    fn add_ival(&mut self, key: String, value: BindingRecord) -> Compile<BindingRecord> {
        match self {
            Self::Root(_, _) => panic!("no ivals at root"),
            Self::Handler(_, ivals) => ivals.add(key, value),
            Self::Do(_, ivals) => ivals.add_do(key, value),
        }
    }
    fn add_export(&mut self, name: String, address: Address) -> Compile<()> {
        match self {
            Self::Root(_, exports) => exports.add(name, address),
            _ => Err(CompileError::InvalidExport(name)),
        }
    }
    fn compile_exports(self) -> CompileIR {
        match self {
            Self::Root(_, exports) => exports.compile(),
            _ => panic!("no exports in handlers"),
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
    pub fn module(module: Vec<Stmt>) -> Compile<Vec<IR>> {
        let mut compiler = Compiler::new();
        let mut out = compiler.body(module)?;
        out.append(compiler.frames.pop().unwrap().compile_exports()?);
        Ok(out.to_vec())
    }
    fn new() -> Self {
        Compiler {
            frames: vec![CompilerFrame::root()],
        }
    }
    pub fn body(&mut self, mut body: Vec<Stmt>) -> CompileIR {
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
    pub fn handler(&mut self, ivals: IVals) {
        self.frames.push(CompilerFrame::handler(ivals))
    }
    pub fn do_handler(&mut self, ivals: IVals) {
        self.frames.push(CompilerFrame::do_handler(ivals))
    }
    pub fn end_handler(&mut self) -> IVals {
        self.frames.pop().unwrap().ivals()
    }
    fn top_mut(&mut self) -> &mut CompilerFrame {
        self.frames.last_mut().unwrap()
    }
    pub fn add_let_export(&mut self, key: String) -> Compile<()> {
        let address = self.top_mut().locals_mut().add_let(key.to_string());
        self.top_mut().add_export(key, address)?;
        Ok(())
    }
    pub fn add_let(&mut self, key: String) {
        self.top_mut().locals_mut().add_let(key);
    }
    pub fn add_anon(&mut self) -> Address {
        self.top_mut().locals_mut().add_anon()
    }
    pub fn add_var(&mut self, key: String) -> CompileIR {
        let index = self.top_mut().locals_mut().add_var(key);
        Ok(IRBuilder::from(vec![IR::Var(index)]))
    }
    pub fn add_var_param(&mut self, key: String) {
        self.top_mut().locals_mut().add_var_param(key);
    }
    pub fn add_do_param(&mut self, key: String) {
        self.top_mut().locals_mut().add_do_param(key);
    }
    pub fn identifier(&mut self, key: String) -> CompileIR {
        self.get(&key)?.identifier(key)
    }
    pub fn target_identifier(&mut self, key: String) -> CompileIR {
        self.get(&key)?.target_identifier(key)
    }
    pub fn arg_identifier(&mut self, key: String) -> CompileIR {
        self.get(&key)?.arg_identifier(key)
    }
    pub fn var_arg(&mut self, key: String) -> CompileIR {
        self.get(&key)?.var_arg(key)
    }
    pub fn set(&mut self, key: String) -> CompileIR {
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
    use crate::{
        ast::{Binding, Expr, Object},
        ir::{Class, Param},
    };

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

    fn let_(binding: Binding, value: Expr) -> Stmt {
        Stmt::Let(binding, value, false)
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
                let_(b_ident("foo"), int(123)),
                let_(b_ident("bar"), int(456)),
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
            vec![IR::Integer(456), IR::Integer(123), IR::send("+:", 1)],
        );
    }

    #[test]
    fn vars() {
        assert_ok(
            vec![
                let_(b_ident("foo"), int(456)),
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
                let_(b_ident("foo"), int(456)),
                Stmt::Set(b_ident("foo"), int(789)),
            ],
            CompileError::InvalidSet("foo".to_string()),
        )
    }

    #[test]
    fn empty_object() {
        assert_ok(
            vec![Stmt::Expr(Expr::Object(Object::new()))],
            vec![IR::Object(Class::new().rc(), 0)],
        )
    }

    #[test]
    fn object_with_simple_handler() {
        assert_ok(
            vec![
                let_(b_ident("foo"), int(123)),
                let_(b_ident("bar"), int(456)),
                Stmt::Expr(Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "handler",
                        vec![],
                        vec![let_(b_ident("bar"), int(789)), Stmt::Expr(ident("bar"))],
                    );
                    obj
                })),
            ],
            vec![
                IR::Integer(123),
                IR::Integer(456),
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add("handler", vec![], vec![IR::Integer(789), IR::Local(0)]);

                        class.rc()
                    },
                    0,
                ),
            ],
        )
    }

    #[test]
    fn object_with_args() {
        assert_ok(
            vec![
                let_(b_ident("foo"), int(123)),
                let_(b_ident("bar"), int(456)),
                Stmt::Expr(Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "handler:",
                        vec![b_ident("foo")],
                        vec![
                            let_(b_ident("bar"), int(789)),
                            Stmt::Expr(send(ident("foo"), "+:", vec![ident("bar")])),
                        ],
                    );
                    obj
                })),
            ],
            vec![
                IR::Integer(123),
                IR::Integer(456),
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add(
                            "handler:",
                            vec![Param::Value],
                            vec![
                                IR::Integer(789),
                                IR::Local(1),
                                IR::Local(0),
                                IR::send("+:", 1),
                            ],
                        );

                        class.rc()
                    },
                    0,
                ),
            ],
        )
    }

    #[test]
    fn instance_values() {
        assert_ok(
            vec![
                let_(b_ident("foo"), int(123)),
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
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add(
                            "handler:",
                            vec![Param::Value],
                            vec![IR::Local(0), IR::IVal(0), IR::send("+:", 1)],
                        );
                        class.rc()
                    },
                    1,
                ),
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
                let_(b_ident("foo"), int(123)),
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
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add(
                            "handler:",
                            vec![Param::Value],
                            vec![IR::Local(0), IR::IVal(0), IR::send("+:", 1)],
                        );
                        class.add("other", vec![], vec![IR::IVal(0)]);

                        class.rc()
                    },
                    1,
                ),
            ],
        )
    }

    #[test]
    fn var_args() {
        assert_ok(
            vec![
                let_(
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
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add(
                            "handler:",
                            vec![Param::Var],
                            vec![
                                IR::SelfRef,
                                IR::Integer(10),
                                IR::Local(0),
                                IR::SetVar,
                                IR::Unit,
                            ],
                        );
                        class.rc()
                    },
                    0,
                ),
                IR::Integer(5),
                IR::Var(1),
                IR::Local(2),
                IR::Local(0),
                IR::send("handler:", 1),
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
                let_(
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
                let_(b_ident("foo"), int(5)),
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
                IR::DoObject(
                    {
                        let mut class = Class::new();
                        class.add("foo", vec![], vec![IR::Integer(123)]);

                        class.rc()
                    },
                    0,
                ),
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add(":", vec![Param::Do], vec![IR::Local(0), IR::send("foo", 0)]);

                        class.rc()
                    },
                    0,
                ),
                IR::send(":", 1),
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
                IR::DoObject(
                    {
                        let mut class = Class::new();
                        class.add(
                            "foo",
                            vec![],
                            vec![IR::Integer(456), IR::IVal(0), IR::SetVar, IR::Unit],
                        );
                        class.rc()
                    },
                    1,
                ),
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add(":", vec![Param::Do], vec![IR::Local(0), IR::send("foo", 0)]);

                        class.rc()
                    },
                    0,
                ),
                IR::send(":", 1),
            ],
        )
    }

    #[test]
    fn invalid_do_arg_reference() {
        assert_err(
            vec![Stmt::Expr(send(
                Expr::Object({
                    let mut object = Object::new();
                    object.add(":", vec![b_do("f")], vec![let_(b_ident("g"), ident("f"))]);
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
            vec![IR::Object(
                {
                    let mut class = Class::new();
                    class.add(
                        ":",
                        vec![Param::Do],
                        vec![
                            IR::Local(0),
                            IR::DoObject(
                                {
                                    let mut class = Class::new();
                                    class.add("foo", vec![], vec![IR::IVal(0), IR::send("foo", 0)]);
                                    class.rc()
                                },
                                1,
                            ),
                            IR::Object(Class::new().rc(), 0),
                            IR::send(":", 1),
                        ],
                    );
                    class.rc()
                },
                0,
            )],
        );
    }

    #[test]
    fn exports() {
        assert_eq!(
            Compiler::module(vec![Stmt::Let(b_ident("foo"), Expr::Integer(123), true),]),
            Ok(vec![
                IR::Integer(123),
                IR::Unit,
                IR::Local(0),
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add("foo", vec![], vec![IR::IVal(0)]);
                        class.rc()
                    },
                    1
                )
            ])
        )
    }

    #[test]
    fn duplicate_export() {
        assert_err(
            vec![
                Stmt::Let(b_ident("foo"), Expr::Integer(123), true),
                Stmt::Let(b_ident("foo"), Expr::Integer(456), true),
            ],
            CompileError::DuplicateExport("foo".to_string()),
        );
    }

    #[test]
    fn invalid_export() {
        assert_err(
            vec![Stmt::Expr(Expr::Object({
                let mut obj = Object::new();
                obj.add(
                    "",
                    vec![],
                    vec![Stmt::Let(b_ident("foo"), Expr::Integer(123), true)],
                );
                obj
            }))],
            CompileError::InvalidExport("foo".to_string()),
        )
    }

    #[test]
    fn destructuring() {
        assert_ok(
            vec![
                Stmt::Let(
                    Binding::Destructure(vec![("get x".to_string(), b_ident("x"))]),
                    Expr::Unit,
                    false,
                ),
                Stmt::Expr(ident("x")),
            ],
            vec![IR::Unit, IR::Local(0), IR::send("get x", 0), IR::Local(1)],
        )
    }

    #[test]
    fn destructuring_param() {
        assert_ok(
            vec![Stmt::Expr(Expr::Object({
                let mut obj = Object::new();
                obj.add(
                    "foo:",
                    vec![Binding::Destructure(vec![(
                        "get x".to_string(),
                        b_ident("x"),
                    )])],
                    vec![Stmt::Expr(ident("x"))],
                );
                obj
            }))],
            vec![IR::Object(
                {
                    let mut class = Class::new();
                    class.add(
                        "foo:",
                        vec![Param::Value],
                        vec![IR::Local(0), IR::send("get x", 0), IR::Local(1)],
                    );

                    class.rc()
                },
                0,
            )],
        )
    }

    #[test]
    fn indirect_self_ref() {
        assert_ok(
            vec![Stmt::Let(
                b_ident("foo"),
                Expr::Object({
                    let mut obj = Object::new();
                    obj.add(
                        "x",
                        vec![],
                        vec![Stmt::Expr(send(ident("foo"), "x", vec![]))],
                    );
                    obj
                }),
                false,
            )],
            vec![
                IR::Object(
                    {
                        let mut class = Class::new();
                        class.add(
                            "x",
                            vec![],
                            vec![IR::SelfRef, IR::Local(0), IR::send("x", 0)],
                        );
                        class.rc()
                    },
                    0,
                ),
                IR::Unit,
            ],
        )
    }
}
