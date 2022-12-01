use crate::class::{Class, Handler as IRHandler, Param as IRParam};
use crate::compiler::{CompileError, CompileResult, Compiler, Instance};
use crate::ir::IR;
use crate::parse_binding::Binding;
use crate::parse_stmt::Stmt;
use crate::parser::{ParseError, ParseResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ObjectBuilder {
    handlers: HashMap<String, Handler>,
    // else_handler: Option<ElseHandler>,
}

impl ObjectBuilder {
    pub fn new() -> Self {
        ObjectBuilder {
            handlers: HashMap::new(),
            // else_handler: None,
        }
    }
    pub fn compile_do(&self, compiler: &mut Compiler) -> Result<(Vec<IR>, Vec<IR>), CompileError> {
        let mut class = Class::new();
        let mut do_instance = compiler.do_instance();

        for (selector, handler) in self.handlers.iter() {
            compiler.do_handler(do_instance);

            let ir_params = Self::compile_params(compiler, handler);
            let body = Compiler::body(&handler.body, compiler)?;
            class.add(selector.clone(), IRHandler::on(ir_params, body));

            do_instance = compiler.end_do_handler();
        }

        let (own_offset, alloc) = compiler.end_do_instance(do_instance);
        let arg = vec![IR::DoBlock {
            class: class.rc(),
            own_offset,
        }];
        Ok((alloc, arg))
    }

    pub fn compile(&self, compiler: &mut Compiler, binding: Option<&Binding>) -> CompileResult {
        let mut class = Class::new();
        let mut instance = Instance::new();

        for (selector, handler) in self.handlers.iter() {
            compiler.handler(instance);

            let ir_params = Self::compile_params(compiler, handler);

            let mut out = Self::compile_self_binding(compiler, binding);
            let mut body = Compiler::body(&handler.body, compiler)?;
            out.append(&mut body);

            class.add(selector.clone(), IRHandler::on(ir_params, out));

            instance = compiler.end_handler();
        }

        let mut out = instance.ivars();
        let arity = out.len();
        out.push(IR::Object(class.rc(), arity));
        Ok(out)
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
                    let record = compiler.add_let(key.to_string());
                    out.push(IR::Assign(record.index));
                }
                _ => {}
            }
        }
        out
    }

    pub fn add_on(&mut self, params_builder: ParamsBuilder, body: Vec<Stmt>) -> ParseResult<()> {
        params_builder.build(self, body)
    }
    fn add_handler(&mut self, handler: Handler) -> ParseResult<()> {
        if self.handlers.contains_key(&handler.selector) {
            todo!("duplicate key parse error")
        }
        self.handlers.insert(handler.selector.clone(), handler);
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Handler {
    selector: String,
    params: Vec<Param>,
    body: Vec<Stmt>,
}

// #[derive(Debug, Clone)]
// struct ElseHandler {}

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
    fn build(self, ob: &mut ObjectBuilder, body: Vec<Stmt>) -> ParseResult<()> {
        match self {
            ParamsBuilder::KeyBuilder(selector) => ob.add_handler(Handler {
                selector,
                params: Vec::new(),
                body,
            }),
            ParamsBuilder::PairBuilder(builder) => builder.build(ob, body),
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
    pub fn add_value(&mut self, key: String, binding: Binding) -> ParseResult<()> {
        if self.params.contains_key(&key) {
            return Err(ParseError::DuplicateKey(key));
        }
        self.params.insert(
            key,
            ParseParam::Param(ParamWithMatch::Param(Param::Value(binding))),
        );
        Ok(())
    }
    pub fn add_var(&mut self, key: String, ident: String) -> ParseResult<()> {
        if self.params.contains_key(&key) {
            return Err(ParseError::DuplicateKey(key));
        }
        self.params.insert(
            key,
            ParseParam::Param(ParamWithMatch::Param(Param::Var(ident))),
        );
        Ok(())
    }
    pub fn add_do(&mut self, key: String, ident: String) -> ParseResult<()> {
        if self.params.contains_key(&key) {
            return Err(ParseError::DuplicateKey(key));
        }
        self.params.insert(
            key,
            ParseParam::Param(ParamWithMatch::Param(Param::Do(ident))),
        );
        Ok(())
    }
    fn build(self, ob: &mut ObjectBuilder, body: Vec<Stmt>) -> ParseResult<()> {
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
