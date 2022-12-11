use std::collections::HashMap;

use crate::{
    compiler::{CompileIR, Compiler},
    object_builder::ObjectBuilder,
    parse_error::ParseError,
    parse_expr::Expr,
    parser::Parse,
    runtime::IR,
    source::Source,
};

#[derive(Debug, Clone)]
pub struct Send {
    selector: String,
    args: Vec<SendArg>,
}

impl Send {
    pub fn compile(self, compiler: &mut Compiler, target: Expr) -> CompileIR {
        let arity = self.args.len();
        let selector = self.selector.to_string();
        let mut out = self.compile_base(compiler, target)?;
        out.push(IR::Send { selector, arity });
        Ok(out)
    }
    pub fn compile_try(self, _: &mut Compiler, _: Expr, _: Expr) -> CompileIR {
        unimplemented!()
        //     let arity = self.args.len();
        //     let selector = self.selector.to_string();
        //     let mut out = self.compile_base(compiler, target)?;

        //     let mut do_block = ObjectBuilder::new();
        //     do_block
        //         .add_on(
        //             ParamsBuilder::key("".to_string()),
        //             vec![Stmt::Expr(or_else)],
        //         )
        //         .unwrap();
        //     let mut ir = do_block.compile(compiler, None)?;
        //     out.append(&mut ir);
        //     out.push(IR::TrySend(selector, arity));
        //     Ok(out)
    }
    fn compile_base(self, compiler: &mut Compiler, target: Expr) -> CompileIR {
        let mut out = Vec::new();
        for arg in self.args {
            match arg {
                SendArg::Value(value) => {
                    let mut result = value.compile(compiler)?;
                    out.append(&mut result);
                }
                SendArg::Var(_) => {
                    unimplemented!();
                }
                SendArg::Do(builder) => {
                    let mut result = builder.compile_do(compiler)?;
                    out.append(&mut result);
                }
            }
        }
        let mut tgt = target.compile(compiler)?;
        out.append(&mut tgt);
        Ok(out)
    }
}

pub struct SendBuilder {
    args: HashMap<String, SendArg>,
}

#[derive(Debug, Clone)]
enum SendArg {
    Value(Expr),
    Var(String),
    Do(ObjectBuilder),
}

impl SendBuilder {
    pub fn new() -> Self {
        SendBuilder {
            args: HashMap::new(),
        }
    }
    pub fn unary_op(operator: String, target: Expr, source: Source) -> Parse<Expr> {
        Ok(Expr::Send(
            Box::new(target),
            Send {
                selector: operator,
                args: vec![],
            },
            source,
        ))
    }

    pub fn binary_op(operator: String, target: Expr, operand: Expr, source: Source) -> Parse<Expr> {
        Ok(Expr::Send(
            Box::new(target),
            Send {
                selector: operator,
                args: vec![SendArg::Value(operand)],
            },
            source,
        ))
    }

    pub fn build_key(self, key: String, target: Expr, source: Source) -> Parse<Expr> {
        if self.args.len() > 0 {
            return ParseError::expected_pair_got_key(&key);
        }
        Ok(Expr::Send(
            Box::new(target),
            Send {
                selector: key,
                args: vec![],
            },
            source,
        ))
    }
    pub fn add_value(&mut self, key: String, value: Expr) -> Parse<()> {
        self.add(key, SendArg::Value(value))
    }
    pub fn add_var(&mut self, key: String, value: String) -> Parse<()> {
        self.add(key, SendArg::Var(value))
    }
    pub fn add_do(&mut self, key: String, value: ObjectBuilder) -> Parse<()> {
        self.add(key, SendArg::Do(value))
    }
    fn add(&mut self, key: String, value: SendArg) -> Parse<()> {
        if self.args.contains_key(&key) {
            return ParseError::duplicate_key(&key);
        }
        self.args.insert(key, value);
        Ok(())
    }
    pub fn build(self, target: Expr, source: Source) -> Parse<Expr> {
        let mut selector = String::new();
        let mut args = Vec::new();

        let mut entries = self.args.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (key, arg) in entries {
            selector.push_str(&key);
            selector.push(':');
            args.push(arg);
        }

        Ok(Expr::Send(
            Box::new(target),
            Send { selector, args },
            source,
        ))
    }
}
