use crate::class::{Class, Param as IRParam};
use crate::compiler::{Compile, CompileIR, Compiler, Instance};
use crate::parse_binding::Binding;
use crate::parse_error::ParseError;
use crate::parse_stmt::Stmt;
use crate::parser::Parse;
use crate::runtime::IR;
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub struct ObjectBuilder {
    handlers: HashMap<String, Handler>,
    else_handler: Option<ElseHandler>,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        ObjectBuilder {
            handlers: HashMap::new(),
            else_handler: None,
        }
    }
    pub fn compile_do(self, compiler: &mut Compiler) -> CompileIR {
        let (class, instance) = self.compile_inner(compiler, None)?;

        let mut out = instance.ivars();
        let arity = out.len();
        out.push(IR::NewDoObject {
            class: class.rc(),
            arity,
        });
        Ok(out)
    }
    pub fn compile(self, compiler: &mut Compiler, binding: Option<&Binding>) -> CompileIR {
        let (class, instance) = self.compile_inner(compiler, binding)?;

        let mut out = instance.ivars();
        let arity = out.len();
        out.push(IR::NewObject {
            class: class.rc(),
            arity,
        });
        Ok(out)
    }
    fn compile_inner(
        self,
        compiler: &mut Compiler,
        binding: Option<&Binding>,
    ) -> Compile<(Class, Instance)> {
        let mut class = Class::new();
        let mut instance = Instance::new();

        for (selector, handler) in self.handlers.into_iter() {
            compiler.handler(instance);

            let ir_params = Self::compile_params(compiler, &handler);

            let mut out = Self::compile_self_binding(compiler, binding);
            let mut body = Compiler::body(handler.body, compiler)?;
            out.append(&mut body);

            class.add_handler(&selector, ir_params, out);

            instance = compiler.end_handler();
        }

        if let Some(else_handler) = self.else_handler {
            compiler.handler(instance);

            let mut out = Self::compile_self_binding(compiler, binding);
            let mut body = Compiler::body(else_handler.body, compiler)?;
            out.append(&mut body);

            class.add_else(out);

            instance = compiler.end_handler()
        }

        Ok((class, instance))
    }
    fn compile_params(compiler: &mut Compiler, handler: &Handler) -> Vec<IRParam> {
        let mut ir_params = Vec::new();
        for param in handler.params.iter() {
            match param {
                Param::Value(binding) => {
                    ir_params.push(IRParam::Value);
                    match binding {
                        Binding::Identifier(key, _) => {
                            compiler.add_let(key.to_string());
                        }
                        Binding::Placeholder(_) => {}
                        Binding::Destructuring(_, _) => todo!("destructuring in params"),
                    }
                }
                Param::Var(key) => {
                    ir_params.push(IRParam::Var);
                    compiler.add_var(key.to_string());
                }
                Param::Do(key) => {
                    ir_params.push(IRParam::Do);
                    compiler.add_let(key.to_string());
                }
            };
        }
        ir_params
    }
    fn compile_self_binding(compiler: &mut Compiler, binding: Option<&Binding>) -> Vec<IR> {
        let mut out = Vec::new();
        if let Some(binding) = binding {
            match binding {
                Binding::Identifier(key, _) => {
                    out.push(IR::SelfRef);
                    compiler.add_let(key.to_string());
                }
                _ => {}
            }
        }
        out
    }

    pub fn add_on(&mut self, params_builder: ParamsBuilder, body: Vec<Stmt>) -> Parse<()> {
        params_builder.build(self, body)
    }
    fn add_handler(&mut self, handler: Handler) -> Parse<()> {
        if self.handlers.contains_key(&handler.selector) {
            return ParseError::duplicate_handler(&handler.selector);
        }
        self.handlers.insert(handler.selector.clone(), handler);
        Ok(())
    }
    pub fn add_else(&mut self, body: Vec<Stmt>) -> Parse<()> {
        if self.else_handler.is_some() {
            return ParseError::duplicate_else_handler();
        }
        self.else_handler = Some(ElseHandler { body });
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Handler {
    selector: String,
    params: Vec<Param>,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct ElseHandler {
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
enum Param {
    Value(Binding),
    Var(String),
    Do(String),
}

#[derive(Debug, Clone)]
pub enum ParamsBuilder {
    KeyBuilder(String),
    PairBuilder(PairParamsBuilder),
}

impl ParamsBuilder {
    pub fn key(key: String) -> Self {
        ParamsBuilder::KeyBuilder(key)
    }
    fn build(self, ob: &mut ObjectBuilder, body: Vec<Stmt>) -> Parse<()> {
        match self {
            ParamsBuilder::KeyBuilder(selector) => ob.add_handler(Handler {
                selector,
                params: Vec::new(),
                body,
            }),
            ParamsBuilder::PairBuilder(builder) => builder.build(ob, body),
        }
    }
    pub fn build_destructuring(self) -> Parse<HashMap<String, Binding>> {
        match self {
            ParamsBuilder::KeyBuilder(_) => todo!("error: cannot destructure with key"),
            ParamsBuilder::PairBuilder(pairs) => {
                let mut out = HashMap::new();
                for (key, param) in pairs.params {
                    if let ParseParam::Param(ParamWithMatch::Param(Param::Value(binding))) = param {
                        out.insert(key, binding);
                    } else {
                        todo!("error: invalid destructuring param")
                    }
                }
                Ok(out)
            }
        }
    }
}

// This is where all the stuff like default & partial params are handled
// these exist at _parse_ time, not compile time
#[derive(Debug, Clone)]
pub struct PairParamsBuilder {
    params: HashMap<String, ParseParam>,
}

impl PairParamsBuilder {
    pub fn new() -> Self {
        PairParamsBuilder {
            params: HashMap::new(),
        }
    }
    pub fn add_value(&mut self, key: String, binding: Binding) -> Parse<()> {
        self.add(
            key,
            ParseParam::Param(ParamWithMatch::Param(Param::Value(binding))),
        )
    }
    pub fn add_var(&mut self, key: String, ident: String) -> Parse<()> {
        self.add(
            key,
            ParseParam::Param(ParamWithMatch::Param(Param::Var(ident))),
        )
    }
    pub fn add_do(&mut self, key: String, ident: String) -> Parse<()> {
        self.add(
            key,
            ParseParam::Param(ParamWithMatch::Param(Param::Do(ident))),
        )
    }
    fn add(&mut self, key: String, param: ParseParam) -> Parse<()> {
        if self.params.contains_key(&key) {
            return ParseError::duplicate_key(&key);
        }
        self.params.insert(key, param);
        Ok(())
    }
    fn build(self, ob: &mut ObjectBuilder, body: Vec<Stmt>) -> Parse<()> {
        for params in self.expand_defaults() {
            let mut selector = String::new();
            let mut out_params = Vec::new();

            let mut entries = params.into_iter().collect::<Vec<_>>();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));

            for (key, param) in entries {
                selector.push_str(&key);
                selector.push(':');
                match param {
                    ParamWithMatch::Param(p) => {
                        out_params.push(p);
                    }
                }
            }

            ob.add_handler(Handler {
                selector,
                params: out_params,
                body: body.clone(),
            })?;
        }

        Ok(())
    }
    fn expand_defaults(self) -> Vec<HashMap<String, ParamWithMatch>> {
        // TODO: each combination of defaults
        let map = self
            .params
            .into_iter()
            .map(|(key, val)| match val {
                ParseParam::Param(val) => (key, val),
            })
            .collect::<HashMap<_, _>>();
        vec![map]
    }
}

#[derive(Debug, Clone)]
enum ParamWithMatch {
    Param(Param),
    // MatchExpr(Expr),
    // MatchParams(Vec<Param>)
}

#[derive(Debug, Clone)]
enum ParseParam {
    Param(ParamWithMatch),
    // DefaultParam(Binding, Expr)
}
