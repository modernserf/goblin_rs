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
fn run(code: &str) -> Result<value::Value, interpreter::RuntimeError> {
    let lexer = lexer::Lexer::from_string(code);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.program().unwrap();
    let ir = compiler::Compiler::program(program).unwrap();
    let result = interpreter::Interpreter::program(ir);
    result
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use crate::interpreter::RuntimeError;
    use crate::run;
    use crate::value::Value;

    #[test]
    fn literals() {
        assert_eq!(run("123").unwrap(), Value::Integer(123));
        assert_eq!(run("1_000").unwrap(), Value::Integer(1000));
    }
    #[test]
    fn assignment() {
        assert_eq!(
            run("
            let x := 2
            let y := 1
            x
        ")
            .unwrap(),
            Value::Integer(2)
        );
    }
    #[test]
    fn unary_operators() {
        assert_eq!(run("-10").unwrap(), Value::Integer(-10));
        assert_eq!(run("- -10").unwrap(), Value::Integer(10));
        assert_eq!(
            run("~~10"),
            Err(RuntimeError::DoesNotUnderstand("~~".to_string()))
        )
    }

    #[test]
    fn binary_operators() {
        assert_eq!(run("1 + 2 + 3").unwrap(), Value::Integer(6));
        assert_eq!(run("1 + 2 + -3").unwrap(), Value::Integer(0));
    }
}
