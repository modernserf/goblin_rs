use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    compiler_2::{CompileIR, Compiler, IRBuilder, IVals},
    parser_2::Parse,
    runtime_2::{Class, Param, Selector, IR},
};

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
    Return(Expr),
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
                ir.push(IR::Unit);
                Ok(ir)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Unit,
    SelfRef,
    Integer(i64),
    Identifier(String),
    Send(Selector, Box<Expr>, Vec<Expr>),
    Object(Object),
    VarArg(String),
    DoArg(Object),
    Frame(Selector, Vec<(String, Expr)>),
}

impl Expr {
    fn compile(self, compiler: &mut Compiler) -> CompileIR {
        match self {
            Self::Unit => Ok(IRBuilder::from(vec![IR::Unit])),
            Self::SelfRef => Ok(IRBuilder::from(vec![IR::SelfRef])),
            Self::Integer(value) => Ok(IRBuilder::from(vec![IR::Integer(value)])),
            Self::Identifier(name) => compiler.identifier(name),
            Self::Send(selector, target, args) => {
                let mut ir = IRBuilder::new();
                let arity = args.len();
                for arg in args {
                    ir.append(arg.compile_arg(compiler)?);
                }
                ir.append(target.compile_target(compiler)?);
                ir.push(IR::Send(selector, arity));
                Ok(ir)
            }
            Self::Object(obj) => obj.compile(compiler),
            Self::Frame(selector, pairs) => {
                let class = frame_class(selector, &pairs);
                let arity = pairs.len();
                let mut ir = IRBuilder::new();
                for (_, expr) in pairs {
                    ir.append(expr.compile_arg(compiler)?);
                }
                ir.push(IR::Object(class, arity));
                Ok(ir)
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
        let arity = ivals.count();
        let mut out = ivals.compile()?;
        out.push(IR::Object(class.rc(), arity));
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
        let arity = ivals.count();
        let mut out = ivals.compile()?;
        out.push(IR::DoObject(class.rc(), arity));
        Ok(out)
    }
}

thread_local! {
    static FRAME_CACHE: RefCell<HashMap<String, Rc<Class>>> = RefCell::new(HashMap::new());
}

pub fn frame_class(selector: String, pairs: &Vec<(String, Expr)>) -> Rc<Class> {
    let cached = FRAME_CACHE.with(|cell| {
        let map = cell.borrow();
        map.get(&selector).cloned()
    });
    if let Some(class) = cached {
        return class;
    }

    let mut class = Class::new();
    // match
    class.add_handler(":".to_string(), vec![Param::Do], {
        let mut builder = IRBuilder::new();
        for i in 0..pairs.len() {
            builder.push(IR::IVal(i));
        }
        builder.push(IR::Local(0));
        builder.push(IR::Send(selector.to_string(), pairs.len()));
        builder.to_vec()
    });

    if pairs.len() == 0 {
        // fold
        class.add_handler(
            ":into:".to_string(),
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
                builder.to_vec()
            });
        }
    }

    let rc = class.rc();

    FRAME_CACHE.with(|cell| {
        let mut map = cell.borrow_mut();
        map.insert(selector, rc.clone());
    });

    rc
}
