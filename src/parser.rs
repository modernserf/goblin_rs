use crate::{
    lexer::{Lexer, Token},
    parse_expr::Expr,
    parse_stmt::Stmt,
};

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
        // TODO: drop whitespace, comments
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
            _ => None,
        }
    }
    fn stmt(&mut self) -> Option<Stmt> {
        let expr = self.base_expr()?;
        Some(Stmt::Expr(expr))
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
    }
}
