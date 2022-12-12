mod class;
mod compiler;
mod frame;
mod ir;
mod lexer;
mod module_loader;
mod object_builder;
mod parse_binding;
mod parse_error;
mod parse_expr;
mod parse_stmt;
mod parser;
mod primitive;
mod runtime;
mod send_builder;
mod source;
mod value;

fn compile_module(code: &str) -> Vec<ir::IR> {
    let lexer = lexer::Lexer::from_string(code);
    let mut parser = parser::Parser::new(lexer);
    let module = parser.program().unwrap();
    compiler::Compiler::module(module).unwrap()
}

fn build_stdlib() -> module_loader::ModuleLoader {
    let mut modules = module_loader::ModuleLoader::new();
    modules.add_ready("native", primitive::native_module());
    modules.add_init("core", compile_module(include_str!("./stdlib/core.gob")));
    modules.add_init(
        "core/option",
        compile_module(include_str!("./stdlib/option.gob")),
    );
    modules.add_init("core/ord", compile_module(include_str!("./stdlib/ord.gob")));
    modules.add_init(
        "core/result",
        compile_module(include_str!("./stdlib/result.gob")),
    );
    modules.add_init(
        "core/control",
        compile_module(include_str!("./stdlib/control.gob")),
    );
    modules.add_init(
        "core/iter",
        compile_module(include_str!("./stdlib/iter.gob")),
    );
    modules
}

thread_local! {
    static STDLIB : module_loader::ModuleLoader = build_stdlib()
}

fn run(code: &str) -> Result<value::Value, runtime::RuntimeError> {
    let lexer = lexer::Lexer::from_string(code);
    let mut parser = parser::Parser::new(lexer);
    let program = parser.program().unwrap();
    let ir = compiler::Compiler::program(program).unwrap();
    let mut modules = STDLIB.with(|m| m.clone());
    let result = runtime::eval_module(ir, &mut modules);
    result
}

fn main() {
    let stdin = std::io::stdin();
    let mut input = String::new();
    loop {
        match stdin.read_line(&mut input) {
            Ok(0) => {
                run(&input).unwrap();
                return;
            }
            Ok(_) => {}
            Err(err) => {
                panic!("{:?}", err)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::run;

    #[test]
    fn empty_program() {
        run("").unwrap();
    }

    #[test]
    fn primitives() {
        run(include_str!("./stdlib/primitive.test.gob")).unwrap();
    }

    #[test]
    fn strings() {
        run(include_str!("./stdlib/string.test.gob")).unwrap();
    }

    #[test]
    fn frames() {
        run(include_str!("./stdlib/frame.test.gob")).unwrap();
    }

    #[test]
    fn do_block() {
        run(include_str!("./stdlib/do_block.test.gob")).unwrap();
    }

    #[test]
    fn option() {
        run(include_str!("./stdlib/option.test.gob")).unwrap();
    }

    #[test]
    fn result() {
        run(include_str!("./stdlib/result.test.gob")).unwrap();
    }

    #[test]
    fn var() {
        run(include_str!("./stdlib/var.test.gob")).unwrap();
    }

    #[test]
    fn control() {
        run(include_str!("./stdlib/control.test.gob")).unwrap();
    }
}
