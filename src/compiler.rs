use std::collections::HashMap;

use crate::{
    ir::IR,
    parse_stmt::Stmt,
    scope::{Scope, ScopeRecord, ScopeType},
    source::Source,
};

pub struct Compiler {
    scope: Scope,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompileError {
    UnknownIdentifier(String, Source),
}

pub type CompileResult = Result<Vec<IR>, CompileError>;

impl Compiler {
    fn new() -> Self {
        Compiler {
            scope: Scope::root(),
        }
    }
    pub fn program(program: Vec<Stmt>) -> CompileResult {
        let mut compiler = Compiler::new();
        let mut out = Vec::new();
        for stmt in program.iter() {
            let mut res = stmt.compile(&mut compiler)?;
            out.append(&mut res)
        }
        Ok(out)
    }
    pub fn get(&self, key: &str) -> Option<ScopeRecord> {
        self.scope.get(key)
    }
    pub fn add_let(&mut self, key: String) -> ScopeRecord {
        self.scope.add(key, ScopeType::Let)
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
}
