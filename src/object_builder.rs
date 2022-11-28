use crate::class::{Class, Handler as IRHandler, Param as IRParam};
use crate::compiler::{CompileResult, Compiler};
use crate::ir::IR;
use crate::parse_binding::Binding;
use crate::parse_stmt::Stmt;
use crate::parser::ParseResult;
use std::collections::HashMap;
use std::rc::Rc;

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
    pub fn compile(&self, compiler: &mut Compiler) -> CompileResult {
        let mut class = Class::new();
        let mut out = compiler.with_instance(|instance| {
            for (selector, handler) in self.handlers.iter() {
                instance.with_handler(|mut handler_compiler| {
                    let mut ir_params = Vec::new();
                    for param in handler.params.iter() {
                        ir_params.push(IRParam::Value);
                        match param {
                            Param::Value(binding) => match binding {
                                Binding::Identifier(key, _) => {
                                    handler_compiler.add_let(key.to_string())
                                }
                            },
                        };
                    }

                    let mut body = Vec::new();
                    for stmt in handler.body.iter() {
                        let mut ir = stmt.compile(&mut handler_compiler)?;
                        body.append(&mut ir);
                    }

                    let handler = IRHandler::OnHandler(ir_params, Rc::new(body));
                    class.add(selector.clone(), handler);
                    Ok(())
                })?;
            }
            Ok(())
        })?;
        let arity = out.len();
        out.push(IR::Object(Rc::new(class), arity));
        Ok(out)
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
    // Var(VarBinding),
    // Do(DoBinding),
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
            todo!("parse error: duplicate handler key")
        }
        self.params.insert(
            key,
            ParseParam::Param(ParamWithMatch::Param(Param::Value(binding))),
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
