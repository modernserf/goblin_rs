use crate::lexer::{Lexer, Token, TokenValue};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Integer {
    value: u64,
    start: usize,
    length: usize,
}

fn integer(value: u64, token: Token) -> Integer {
    Integer {
        value,
        start: token.start,
        length: token.length,
    }
}

trait Expr {}

impl Expr for Integer {}

trait Stmt {}

impl<T: Expr> Stmt for T {}

type TokenIter<'a> = std::iter::Peekable<Lexer<'a>>;

pub struct Parser<'a> {
    tokens: TokenIter<'a>,
    eof: Token,
}

impl<'a> Parser<'a> {
    fn new(lexer: Lexer<'a>) -> Parser<'a> {
        Parser {
            tokens: lexer.peekable(),
            eof: Token::eof(),
        }
    }
    fn peek(&mut self) -> &Token {
        // TODO: drop
        self.tokens.peek().unwrap_or(&self.eof)
    }
    fn next(&mut self) -> Token {
        self.tokens.next().unwrap_or(Token::eof())
    }
    fn base_expr(&mut self) -> Option<impl Expr> {
        match self.peek().value {
            TokenValue::Integer(value) => {
                let token = self.next();
                Some(integer(value, token))
            }
            _ => None,
        }
    }
    fn stmt(&mut self) -> Option<impl Stmt> {
        self.base_expr()
    }
    fn program(&mut self) -> Option<Vec<impl Stmt>> {
        let mut out = Vec::new();
        while let Some(stmt) = self.stmt() {
            out.push(stmt)
        }
        if self.next().value == TokenValue::EndOfInput {
            return Some(out);
        }
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn parse(string: &str) -> Option<Vec<impl Stmt>> {
        let lexer = Lexer::from_string(&string);
        Parser::new(lexer).program()
    }

    #[test]
    fn numbers() {
        assert!(parse("0").is_some());
        assert!(parse("123_45").is_some());
    }
}
