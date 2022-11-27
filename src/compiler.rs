use crate::{ir::IR, parse_stmt::Stmt};

pub struct Compiler {}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CompileError {}

pub type CompileResult = Result<Vec<IR>, CompileError>;

impl Compiler {
    pub fn program(program: Vec<Stmt>) -> CompileResult {
        let mut compiler = Compiler {};
        let mut out = Vec::new();
        for stmt in program.iter() {
            let mut res = stmt.compile(&mut compiler)?;
            out.append(&mut res)
        }
        Ok(out)
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
