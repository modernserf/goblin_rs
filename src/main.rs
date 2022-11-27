mod compiler;
mod interpreter;
mod ir;
mod lexer;
mod parse_binding;
mod parse_expr;
mod parse_stmt;
mod parser;
mod scope;
mod source;
mod value;

#[allow(unused)]
fn run(code: &str) -> value::Value {
    let lexer = lexer::Lexer::from_string(code);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.program().unwrap();
    let ir = compiler::Compiler::program(program).unwrap();
    let result = interpreter::Interpreter::program(ir).unwrap();
    result
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use crate::run;
    use crate::value::Value;

    #[test]
    fn smoke_test() {
        assert_eq!(run("123"), Value::Integer(123));
        assert_eq!(run("1_000"), Value::Integer(1000));
        assert_eq!(
            run("
            let x := 2
            let y := 1
            x
        "),
            Value::Integer(2)
        );
    }
}
