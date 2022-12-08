use std::collections::HashMap;

use primitive::native_module;

mod class;
mod compiler;
mod frame;
mod interpreter;
mod ir;
mod lexer;
mod object_builder;
mod parse_binding;
mod parse_error;
mod parse_expr;
mod parse_stmt;
mod parser;
mod primitive;
mod runtime_error;
mod send_builder;
mod source;
mod value;

#[allow(unused)]
fn run(code: &str) -> Result<value::Value, runtime_error::RuntimeError> {
    let lexer = lexer::Lexer::from_string(code);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.program().unwrap();
    let ir = compiler::Compiler::program(program).unwrap();
    let mut modules = HashMap::new();
    modules.insert("native".to_string(), native_module());
    let result = interpreter::program(ir, modules);
    result
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use crate::run;
    use crate::runtime_error::RuntimeError;
    use crate::value::Value;

    fn assert_ok(code: &str, value: Value) {
        assert_eq!(run(code), Ok(value))
    }
    fn assert_err(code: &str, err: RuntimeError) {
        assert_eq!(run(code), Err(err))
    }

    #[test]
    fn test_file() {
        let file = include_str!("test.gob");
        run(file).unwrap();
    }

    #[test]
    fn empty_program() {
        assert_ok("", Value::Unit)
    }

    #[test]
    fn unit() {
        assert_ok("()", Value::Unit);
    }

    #[test]
    fn literals() {
        assert_ok("123", Value::Integer(123));
        assert_ok("1_000", Value::Integer(1000));
    }
    #[test]
    fn assignment() {
        assert_ok(
            "
            let x := 2
            let y := 1
            x
        ",
            Value::Integer(2),
        );
    }
    #[test]
    fn unary_operators() {
        assert_ok("-10", Value::Integer(-10));
        assert_ok("- -10", Value::Integer(10));
        assert_err("~~10", RuntimeError::DoesNotUnderstand("~~".to_string()))
    }

    #[test]
    fn binary_operators() {
        assert_ok("1 + 2 + 3", Value::Integer(6));
        assert_ok("1 + 2 + -3", Value::Integer(0));
    }

    #[test]
    fn bools() {
        assert_ok("1 = 1", Value::Bool(true));
    }

    #[test]
    fn floats() {
        assert_ok("1 + 2.0 + 3", Value::Float(6.0));
    }

    #[test]
    fn strings() {
        assert_ok("\"hello\" ++ \" world\"", Value::string("hello world"));
    }

    #[test]
    fn parens() {
        assert_ok("1 + (2 + 3)", Value::Integer(6));
        assert_err(
            "1 + ()",
            RuntimeError::PrimitiveTypeError {
                expected: "number".to_string(),
                received: Value::Unit,
            },
        )
    }
    #[test]
    fn send() {
        assert_ok("10{-}", Value::Integer(-10));
        assert_ok("1{+: 2}{+: 3}", Value::Integer(6));
    }

    #[test]
    fn object() {
        assert_ok(
            "
            let x := [on {foo}]
            x{foo}
        ",
            Value::Unit,
        );
        assert_ok(
            "
            let x := [
                on {} 1
                on {foo} 2
                on {bar: arg} arg
            ]
            let bar := 3
            x{} + x{foo} + x{bar: bar}
        ",
            Value::Integer(6),
        );
    }
    #[test]
    fn closure() {
        assert_ok(
            "
            let foo := 2
            let x := [
                on {} 1
                on {foo} foo
                on {bar: arg} arg
            ]
            let bar := 3
            x{} + x{foo} + x{bar: bar}
        ",
            Value::Integer(6),
        );
    }

    #[test]
    fn frame() {
        assert_ok(
            "
            let point := [x: 1 y: 2]
            point{x} + point{y}
        ",
            Value::Integer(3),
        );
        assert_ok(
            "
            let point := [x: 1 y: 2]
            let other := point{x: 2}
            point{x} + other{x}
        ",
            Value::Integer(3),
        );
    }

    #[test]
    fn self_ref() {
        assert_ok(
            "
            let target := [
                on {x}
                    self{y}
                on {y}
                    2
            ]
            target{x}
        ",
            Value::Integer(2),
        );
    }

    #[test]
    fn indirect_self_ref() {
        assert_ok(
            "
            let Point := [
                on {x: x y: y} [
                    on {x}
                        x
                    on {x: x'}
                        Point{x: x' y: y}
                ]
            ]
            let p := Point{x: 1 y: 2}
            let q := p{x: 3}
            q{x}
        ",
            Value::Integer(3),
        )
    }

    #[test]
    fn do_blocks() {
        assert_ok(
            "
            let target := [
                on {foo: do f}
                    f{bar}
            ]
            let res := target{foo:
                on {bar} 1
            }
            res
        ",
            Value::Integer(1),
        )
    }
}
