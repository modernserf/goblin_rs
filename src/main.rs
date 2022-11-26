mod lexer;
mod parser;

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
