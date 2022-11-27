use crate::{
    lexer::{Lexer, Token},
    parse_binding::Binding,
    parse_expr::Expr,
    parse_stmt::Stmt,
};
use std::mem;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    ExpectedEndOfInput,
    Expected(String),
}

fn expect<T>(value: ParseOpt<T>, error_msg: &str) -> ParseResult<T> {
    match value? {
        Some(x) => Ok(x),
        None => Err(ParseError::Expected(error_msg.to_string())),
    }
}

type ParseResult<T> = Result<T, ParseError>;
type ParseOpt<T> = Result<Option<T>, ParseError>;

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
        self.tokens.peek_mut().unwrap_or(&mut self.eof)
    }
    fn advance(&mut self) -> Token {
        self.tokens.next().unwrap_or(Token::EndOfInput)
    }
    fn accept_token(&mut self, f: fn(t: &Token) -> bool) -> bool {
        if f(self.peek()) {
            self.advance();
            return true;
        }
        return false;
    }
    fn expect_token(&mut self, error_msg: &str, f: fn(t: &Token) -> bool) -> ParseResult<()> {
        if f(self.peek()) {
            self.advance();
            return Ok(());
        }
        return Err(ParseError::Expected(error_msg.to_string()));
    }
    fn base_expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::Integer(value, source) => {
                let val = *value;
                let src = *source;
                self.advance();
                Ok(Some(Expr::Integer(val, src)))
            }
            Token::Identifier(value, source) => {
                let key = mem::take(value);
                let src = *source;
                self.advance();
                Ok(Some(Expr::Identifier(key, src)))
            }
            _ => Ok(None),
        }
    }
    fn unary_op_expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::Operator(value, source) => {
                let src = *source;
                let selector = mem::take(value);
                self.advance();
                let expr = expect(self.unary_op_expr(), "expr")?;
                Ok(Some(Expr::UnaryOp(selector, Box::new(expr), src)))
            }
            _ => self.base_expr(),
        }
    }
    fn binary_op_expr(&mut self) -> ParseOpt<Expr> {
        let mut expr = match self.unary_op_expr()? {
            Some(expr) => expr,
            None => return Ok(None),
        };
        while let Token::Operator(value, source) = self.peek() {
            let src = *source;
            let mut selector = mem::take(value);
            selector.push(':');
            self.advance();
            let operand = expect(self.unary_op_expr(), "expr")?;
            expr = Expr::BinaryOp {
                selector,
                target: Box::new(expr),
                operand: Box::new(operand),
                source: src,
            }
        }
        Ok(Some(expr))
    }

    fn expr(&mut self) -> ParseOpt<Expr> {
        self.binary_op_expr()
    }

    fn binding(&mut self) -> ParseOpt<Binding> {
        match self.peek() {
            Token::Identifier(value, source) => {
                let key = mem::take(value);
                let src = *source;
                self.advance();
                Ok(Some(Binding::Identifier(key, src)))
            }
            _ => Ok(None),
        }
    }
    fn stmt(&mut self) -> ParseOpt<Stmt> {
        match self.peek() {
            Token::Let(_) => {
                self.advance();
                let binding = expect(self.binding(), "binding")?;
                self.expect_token("colon equals", |t| match t {
                    Token::ColonEquals(_) => true,
                    _ => false,
                })?;
                let expr = expect(self.expr(), "expr")?;

                Ok(Some(Stmt::Let(binding, expr)))
            }
            _ => match self.expr()? {
                Some(expr) => Ok(Some(Stmt::Expr(expr))),
                None => Ok(None),
            },
        }
    }
    pub fn program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut out = Vec::new();
        while let Some(stmt) = self.stmt()? {
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
