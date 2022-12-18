use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    compiler::{CompileIR, Compiler, IRBuilder, IVals},
    ir::{Address, Class, Handler as IRHandler, Param, Selector, Value, IR},
    native::{bool_class, int_class, string_class},
    parser::{Parse, ParseError},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Binding {
    Identifier(String),
    VarIdentifier(String),
    DoIdentifier(String),
    Destructure(Vec<(String, Binding)>),
}
impl Binding {
    fn compile_let(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Identifier(name) => {
                compiler.add_let(name);
                Ok(IRBuilder::new())
            }
            Self::Destructure(items) => {
                let addr = compiler.add_anon();
                let mut ir = IRBuilder::new();
                for (key, binding) in items {
                    ir.push(IR::Local(addr));
                    ir.push(IR::Send(key, 0));
                    ir.append(binding.compile_let(compiler)?);
                }
                Ok(ir)
            }
            _ => todo!(),
        }
    }
    fn compile_export(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Identifier(name) => {
                compiler.add_let_export(name)?;
                Ok(IRBuilder::new())
            }
            Self::Destructure(items) => {
                let addr = compiler.add_anon();
                let mut ir = IRBuilder::new();
                for (key, binding) in items {
                    ir.push(IR::Local(addr));
                    ir.push(IR::Send(key, 0));
                    ir.append(binding.compile_export(compiler)?);
                }
                Ok(ir)
            }
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
    fn compile_param(self, compiler: &mut Compiler) -> ParamResult {
        match self {
            Self::Identifier(name) => {
                compiler.add_let(name);
                ParamResult::Value
            }
            Self::Destructure(items) => {
                let addr = compiler.add_anon();
                ParamResult::Destructure(addr, items)
            }
            Self::VarIdentifier(name) => {
                compiler.add_var_param(name);
                ParamResult::Var
            }
            Self::DoIdentifier(name) => {
                compiler.add_do_param(name);
                ParamResult::Do
            }
        }
    }
}

pub enum ParamResult {
    Value,
    Destructure(Address, Vec<(String, Binding)>),
    Var,
    Do,
}
impl ParamResult {
    pub fn param(&self) -> Param {
        match self {
            Self::Value => Param::Value,
            Self::Destructure(_, _) => Param::Value,
            Self::Var => Param::Var,
            Self::Do => Param::Do,
        }
    }
    pub fn compile(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Destructure(addr, items) => {
                let mut ir = IRBuilder::new();
                for (key, binding) in items {
                    ir.push(IR::Local(addr));
                    ir.push(IR::Send(key, 0));
                    ir.append(binding.compile_let(compiler)?);
                }
                Ok(ir)
            }
            _ => Ok(IRBuilder::new()),
        }
    }
}

type IsExport = bool;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Let(Binding, Expr, IsExport),
    Var(Binding, Expr),
    Set(Binding, Expr),
    Import(Binding, String, IsExport),
    Return(Expr),
}

impl Stmt {
    fn compile_base(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Expr(expr) => expr.compile(compiler),
            Self::Let(binding, expr, is_export) => {
                let mut ir = expr.compile_with_binding(compiler, &binding)?;
                if is_export {
                    ir.append(binding.compile_export(compiler)?);
                } else {
                    ir.append(binding.compile_let(compiler)?);
                }
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
            Self::Import(binding, name, is_export) => {
                let mut ir = IRBuilder::from(vec![IR::Module(name)]);
                if is_export {
                    ir.append(binding.compile_export(compiler)?);
                } else {
                    ir.append(binding.compile_let(compiler)?);
                }
                Ok(ir)
            }
            Self::Return(expr) => {
                let mut ir = expr.compile(compiler)?;
                ir.push(IR::Return);
                Ok(ir)
            }
        }
    }
    // remove unused stack values
    pub fn compile_most(self, compiler: &mut Compiler) -> CompileIR {
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
    pub fn compile_last(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Expr(_) => self.compile_base(compiler),
            _ => {
                let mut ir = self.compile_base(compiler)?;
                ir.push(IR::unit());
                Ok(ir)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Unit,
    SelfRef,
    Bool(bool),
    Integer(i64),
    String(String),
    Identifier(String),
    Send(Selector, Box<Expr>, Vec<Expr>),
    TrySend(Selector, Box<Expr>, Vec<Expr>, Box<Expr>),
    Object(Object),
    VarArg(String),
    DoArg(Object),
    Frame(Selector, Vec<(String, Expr)>),
    If(Box<Expr>, Vec<Stmt>, Vec<Stmt>),
    Paren(Vec<Stmt>),
}

impl Expr {
    fn send(selector: &str, target: Expr, args: Vec<Expr>) -> Self {
        Self::Send(selector.to_string(), Box::new(target), args)
    }
    fn compile(self, compiler: &mut Compiler) -> CompileIR {
        self.compile_base(compiler, None)
    }
    fn compile_with_binding(self, compiler: &mut Compiler, binding: &Binding) -> CompileIR {
        self.compile_base(compiler, Some(binding))
    }
    fn get_direct_handler(&self, _: &str) -> Option<Rc<IRHandler>> {
        match self {
            // Self::Integer(_) => int_class().get(selector).ok(),
            // Self::Bool(_) => bool_class().get(selector).ok(),
            // Self::String(_) => string_class().get(selector).ok(),
            _ => None,
        }
    }
    fn compile_send(&self, selector: String, arity: usize) -> CompileIR {
        match self.get_direct_handler(&selector) {
            Some(_) => todo!(),
            // Some(handler) => Ok(IRBuilder::from(vec![IR::SendDirect(handler, arity)])),
            None => Ok(IRBuilder::from(vec![IR::Send(selector, arity)])),
        }
    }

    fn compile_base(self, compiler: &mut Compiler, binding: Option<&Binding>) -> CompileIR {
        match self {
            Self::Unit => Ok(IRBuilder::from(vec![IR::unit()])),
            Self::SelfRef => Ok(IRBuilder::from(vec![IR::SelfRef])),
            Self::Bool(value) => Ok(IRBuilder::from(vec![IR::bool(value)])),
            Self::Integer(value) => Ok(IRBuilder::from(vec![IR::int(value)])),
            Self::String(str) => Ok(IRBuilder::from(vec![IR::string(str)])),
            Self::Identifier(name) => compiler.identifier(name),
            Self::Send(selector, target, args) => {
                let mut ir = IRBuilder::new();
                let arity = args.len();
                for arg in args {
                    ir.append(arg.compile_arg(compiler)?);
                }

                let send = target.compile_send(selector, arity)?;
                ir.append(target.compile_target(compiler)?);
                ir.append(send);
                Ok(ir)
            }
            Self::TrySend(selector, target, args, or_else) => {
                let mut ir = IRBuilder::new();
                let arity = args.len();
                for arg in args {
                    ir.append(arg.compile_arg(compiler)?);
                }
                ir.append(
                    Self::DoArg({
                        let mut obj = Object::new();
                        obj.add("", vec![], vec![Stmt::Expr(*or_else)]);
                        obj
                    })
                    .compile_arg(compiler)?,
                );
                ir.append(target.compile_target(compiler)?);
                ir.push(IR::TrySend(selector, arity));
                Ok(ir)
            }
            Self::Object(obj) => obj.compile(compiler, binding),
            Self::Frame(selector, pairs) => {
                let class = frame_class(selector, &pairs);
                let arity = pairs.len();
                let mut ir = IRBuilder::new();
                for (_, expr) in pairs {
                    ir.append(expr.compile_arg(compiler)?);
                }
                ir.push(IR::object(class, arity));
                Ok(ir)
            }
            Self::If(cond, if_true, if_false) => Self::send(
                ":",
                *cond,
                vec![Self::DoArg({
                    let mut obj = Object::new();
                    obj.add("true", vec![], if_true);
                    obj.add("false", vec![], if_false);
                    obj
                })],
            )
            .compile(compiler),
            Self::Paren(body) => {
                if body.is_empty() {
                    return Ok(IRBuilder::from(vec![IR::unit()]));
                }
                if body.len() == 1 {
                    if let Stmt::Expr(expr) = &body[0] {
                        return expr.clone().compile(compiler);
                    }
                }
                Self::send(
                    "",
                    {
                        let mut obj = Object::new();
                        obj.add("", vec![], body);
                        Expr::DoArg(obj)
                    },
                    vec![],
                )
                .compile(compiler)
            }
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
    pub fn as_binding(self) -> Parse<Binding> {
        match self {
            Self::Identifier(name) => Ok(Binding::Identifier(name)),
            _ => Err(ParseError::expected("set binding")),
        }
    }
    pub fn set_target(&self) -> Parse<Binding> {
        match self {
            Self::Identifier(name) => Ok(Binding::Identifier(name.to_string())),
            Self::Send(_, target, _) => target.set_target(),
            _ => Err(ParseError::expected("set target")),
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
    pub fn add(&mut self, selector: &str, params: Vec<Binding>, body: Vec<Stmt>) {
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
    fn compile(self, compiler: &mut Compiler, binding: Option<&Binding>) -> CompileIR {
        let mut class = Class::new();
        let mut ivals = IVals::new();

        for (selector, handler) in self.handlers {
            compiler.handler(ivals);
            let mut param_results = vec![];
            for param in handler.params {
                param_results.push(param.compile_param(compiler));
            }
            let params = param_results.iter().map(|p| p.param()).collect();

            let mut ir = IRBuilder::new();

            for res in param_results {
                ir.append(res.compile(compiler)?);
            }

            if let Some(b) = binding {
                ir.push(IR::SelfRef);
                ir.append(b.clone().compile_let(compiler)?);
            }

            ir.append(compiler.body(handler.body)?);

            class.add_handler(selector, params, ir.build());
            ivals = compiler.end_handler();
        }
        let arity = ivals.count();
        let mut out = ivals.compile()?;
        out.push(IR::object(class.rc(), arity));
        Ok(out)
    }
    fn compile_do(self, compiler: &mut Compiler) -> CompileIR {
        let mut class = Class::new();
        let mut ivals = IVals::new();
        for (selector, handler) in self.handlers {
            compiler.do_handler(ivals);
            let mut param_results = vec![];
            for param in handler.params {
                param_results.push(param.compile_param(compiler));
            }
            let params = param_results.iter().map(|p| p.param()).collect();

            let mut ir = IRBuilder::new();
            for res in param_results {
                ir.append(res.compile(compiler)?);
            }
            ir.append(compiler.body(handler.body)?);

            class.add_handler(selector, params, ir.build());
            ivals = compiler.end_handler();
        }
        let arity = ivals.count();
        let mut out = ivals.compile()?;
        out.push(IR::DoObject(class.rc(), arity));
        Ok(out)
    }
}

thread_local! {
    static FRAME_CACHE: RefCell<HashMap<String, Rc<Class>>> = RefCell::new(HashMap::new());
}

pub fn frame_class(selector: String, pairs: &[(String, Expr)]) -> Rc<Class> {
    let cached = FRAME_CACHE.with(|cell| {
        let map = cell.borrow();
        map.get(&selector).cloned()
    });
    if let Some(class) = cached {
        return class;
    }

    let mut class = Class::new();
    // match
    class.add(":", vec![Param::Do], {
        let mut builder = IRBuilder::new();
        for i in 0..pairs.len() {
            builder.push(IR::IVal(i));
        }
        builder.push(IR::Local(0));
        builder.push(IR::Send(selector.to_string(), pairs.len()));
        builder.build()
    });

    // equality
    // TODO: is this "cheating"
    class.add_native("=:", vec![Param::Value], |target, args| {
        Ok(Value::Bool(target == args[0]))
    });
    class.add_native("!=:", vec![Param::Value], |target, args| {
        Ok(Value::Bool(target != args[0]))
    });

    if pairs.is_empty() {
        // fold
        class.add(
            ":into:",
            vec![Param::Value, Param::Value],
            vec![IR::Send(format!("{}:", &selector), 1)],
        );
    } else {
        for (i, (key, _)) in pairs.iter().enumerate() {
            // getter
            class.add_handler(key.to_string(), vec![], vec![IR::IVal(i)]);
            // setter
            class.add_handler(format!("{}:", &key), vec![Param::Value], {
                let mut builder = IRBuilder::new();
                for j in 0..pairs.len() {
                    if i == j {
                        builder.push(IR::Local(0));
                    } else {
                        builder.push(IR::IVal(j));
                    }
                }
                builder.push(IR::NewSelf(pairs.len()));
                builder.build()
            });
            // update
            class.add_handler(
                format!("-> {}:", &key),
                vec![Param::Do],
                vec![
                    IR::IVal(i),
                    IR::Local(0),
                    IR::send(":", 1),
                    IR::SelfRef,
                    IR::Send(format!("{}:", &key), 1),
                ],
            );
        }
    }

    let rc = class.rc();

    FRAME_CACHE.with(|cell| {
        let mut map = cell.borrow_mut();
        map.insert(selector, rc.clone());
    });

    rc
}
