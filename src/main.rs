mod class;
mod compiler;
mod frame;
mod interpreter;
mod ir;
mod lexer;
mod object_builder;
mod parse_binding;
mod parse_expr;
mod parse_stmt;
mod parser;
mod primitive;
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
    fn empty_program() {
        assert_eq!(run(""), Ok(Value::Unit));
    }

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

    #[test]
    fn floats() {
        assert_eq!(run("1 + 2.0 + 3").unwrap(), Value::Float(6.0));
    }

    #[test]
    fn strings() {
        assert_eq!(
            run("\"hello\" ++ \" world\"").unwrap(),
            Value::string("hello world")
        );
    }

    #[test]
    fn parens() {
        assert_eq!(run("1 + (2 + 3)").unwrap(), Value::Integer(6));
        assert_eq!(
            run("1 + ()"),
            Err(RuntimeError::PrimitiveTypeError {
                expected: "number".to_string(),
                received: Value::Unit
            })
        )
    }
    #[test]
    fn send() {
        assert_eq!(run("10{-}").unwrap(), Value::Integer(-10));
        assert_eq!(run("1{+: 2}{+: 3}").unwrap(), Value::Integer(6));
    }

    #[test]
    fn object() {
        assert_eq!(
            run("
            let x := [
                on {} 1
                on {foo} 2
                on {bar: arg} arg
            ]
            let bar := 3
            x{} + x{foo} + x{bar: bar}
        ")
            .unwrap(),
            Value::Integer(6)
        );
        assert_eq!(
            run("
                let x := 1
                let y := 2
                let target := [
                    on {foo: x}
                        let y := 3
                        x + y
                ]
                let res := target{foo: 10}
                res + x + y
            ")
            .unwrap(),
            Value::Integer(16)
        )
    }
    #[test]
    fn closure() {
        assert_eq!(
            run("
            let foo := 2
            let x := [
                on {} 1
                on {foo} foo
                on {bar: arg} arg
            ]
            let bar := 3
            x{} + x{foo} + x{bar: bar}
        ")
            .unwrap(),
            Value::Integer(6)
        );
    }

    #[test]
    fn frame() {
        assert_eq!(
            run("
                let point := [x: 1 y: 2]
                point{x} + point{y}
            ")
            .unwrap(),
            Value::Integer(3)
        );
        assert_eq!(
            run("
                let point := [x: 1 y: 2]
                let other := point{x: 2}
                point{x} + other{x}
            ")
            .unwrap(),
            Value::Integer(3)
        );
    }
}
