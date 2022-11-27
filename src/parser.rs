use crate::{
    lexer::{Lexer, Token},
    parse_binding::Binding,
    parse_expr::Expr,
    parse_stmt::Stmt,
};
use std::mem;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParseError {
    ExpectedEndOfInput,
}

type TokenIter<'a> = std::iter::Peekable<Lexer<'a>>;

pub struct Parser<'a> {
    tokens: TokenIter<'a>,
    eof: Token,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Parser<'a> {
        Parser {
            tokens: lexer.peekable(),
            eof: Token::EndOfInput,
        }
    }
    fn peek(&mut self) -> &mut Token {
        // drop non-semantic tokens
        loop {
            match self.tokens.peek() {
                Some(Token::Whitespace(_)) => {
                    self.advance();
                }
                Some(Token::Comment(_, _)) => {
                    self.advance();
                }
                _ => break,
            }
        }
        self.tokens.peek_mut().unwrap_or(&mut self.eof)
    }
    fn advance(&mut self) -> Token {
        self.tokens.next().unwrap_or(Token::EndOfInput)
    }
    fn base_expr(&mut self) -> Option<Expr> {
        match self.peek() {
            Token::Integer(value, source) => {
                let val = *value;
                let src = *source;
                self.advance();
                Some(Expr::Integer(val, src))
            }
            Token::Identifier(value, source) => {
                let key = mem::take(value);
                let src = *source;
                self.advance();
                Some(Expr::Identifier(key, src))
            }
            _ => None,
        }
    }
    fn expr(&mut self) -> Option<Expr> {
        self.base_expr()
    }

    fn binding(&mut self) -> Option<Binding> {
        match self.peek() {
            Token::Identifier(value, source) => {
                let key = mem::take(value);
                let src = *source;
                self.advance();
                Some(Binding::Identifier(key, src))
            }
            _ => None,
        }
    }
    fn stmt(&mut self) -> Option<Stmt> {
        match self.peek() {
            Token::Let(_) => {
                self.advance();
                // TODO: parse errors
                let binding = self.binding().unwrap();
                match self.peek() {
                    Token::ColonEquals(_) => self.advance(),
                    _ => unimplemented!(),
                };
                let expr = self.expr().unwrap();

                Some(Stmt::Let(binding, expr))
            }
            _ => {
                let expr = self.expr()?;
                Some(Stmt::Expr(expr))
            }
        }
    }
    pub fn program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut out = Vec::new();
        while let Some(stmt) = self.stmt() {
            out.push(stmt)
        }
        if self.advance() == Token::EndOfInput {
            return Ok(out);
        }
        Err(ParseError::ExpectedEndOfInput)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn parse(string: &str) -> Result<Vec<Stmt>, ParseError> {
        let lexer = Lexer::from_string(&string);
        Parser::new(lexer).program()
    }

    #[test]
    fn numbers() {
        assert!(parse("0").is_ok());
        assert!(parse("123_45").is_ok());
        assert!(parse(
            "123 # a comment
            123_45
            "
        )
        .is_ok())
    }
}
