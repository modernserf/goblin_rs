mod compiler;
mod ir;
mod lexer;
mod parse_expr;
mod parse_stmt;
mod parser;
mod source;

#[allow(unused)]
fn run(code: &str) {
    let lexer = lexer::Lexer::from_string(code);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.program().unwrap();
    let ir = compiler::Compiler::program(program).unwrap();
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    #[test]
    fn smoke_test() {
        assert_eq!(1, 1)
    }
}
