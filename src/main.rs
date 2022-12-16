mod ast;
mod compiler;
mod lexer;
mod native;
mod parser;
mod runtime;

fn compile_module(code: &str) -> Vec<runtime::IR> {
    let tokens = lexer::Lexer::lex(code.to_string());
    let ast = parser::Parser::parse(tokens).unwrap();
    compiler::Compiler::module(ast).unwrap()
}

fn build_stdlib() -> runtime::ModuleLoader {
    let mut modules = runtime::ModuleLoader::new();
    modules.add_ready("native", native::native_module());
    modules.add_init("core", compile_module(include_str!("./stdlib/core.gob")));
    modules.add_init("core/ord", compile_module(include_str!("./stdlib/ord.gob")));
    modules.add_init(
        "core/option",
        compile_module(include_str!("./stdlib/option.gob")),
    );
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
    modules.add_init(
        "core/sortable",
        compile_module(include_str!("./stdlib/sortable.gob")),
    );
    modules.add_init(
        "core/slice",
        compile_module(include_str!("./stdlib/slice.gob")),
    );
    modules.add_init("parse", compile_module(include_str!("./stdlib/parse.gob")));
    modules
}

thread_local! {
    static STDLIB : runtime::ModuleLoader = build_stdlib()
}

fn run(code: &str) {
    let tokens = lexer::Lexer::lex(code.to_string());
    let ast = parser::Parser::parse(tokens).unwrap();
    let ir = compiler::Compiler::program(ast).unwrap();
    let mut modules = STDLIB.with(|m| m.clone());
    let result = runtime::Interpreter::program(ir, &mut modules);
    match result {
        Ok(value) => {
            println!("{:?}", value);
        }
        Err(error) => {
            println!("Error: {:#?}", error);
            panic!("error");
        }
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut input = String::new();
    loop {
        match stdin.read_line(&mut input) {
            Ok(0) => {
                run(&input);
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
        run("")
    }

    #[test]
    fn syntax() {
        run(include_str!("./syntax.gob"))
    }
    #[test]
    fn bool() {
        run(include_str!("./stdlib/bool.test.gob"))
    }
    #[test]
    fn ord() {
        run(include_str!("./stdlib/ord.test.gob"));
    }
    #[test]
    fn option() {
        run(include_str!("./stdlib/option.test.gob"));
    }

    #[test]
    fn strings() {
        run(include_str!("./stdlib/string.test.gob"));
    }

    #[test]
    fn frames() {
        run(include_str!("./stdlib/frame.test.gob"));
    }

    // #[test]
    // fn do_block() {
    //     run(include_str!("./stdlib/do_block.test.gob"));
    // }

    #[test]
    fn result() {
        run(include_str!("./stdlib/result.test.gob"));
    }

    // #[test]
    // fn var() {
    //     run(include_str!("./stdlib/var.test.gob"));
    // }

    #[test]
    fn control() {
        run(include_str!("./stdlib/control.test.gob"));
    }

    #[test]
    fn iter() {
        run(include_str!("./stdlib/iter.test.gob"));
    }

    #[test]
    fn slice() {
        run(include_str!("./stdlib/slice.test.gob"));
    }

    #[test]
    fn parse() {
        run(include_str!("./stdlib/parse.test.gob"));
    }
}
