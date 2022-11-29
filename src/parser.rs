use crate::object_builder::{ObjectBuilder, PairParamsBuilder, ParamsBuilder};
use crate::{
    lexer::{Lexer, Token},
    parse_binding::Binding,
    parse_expr::{Expr, SendBuilder},
    parse_stmt::Stmt,
    source::Source,
};
use std::mem;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    ExpectedEndOfInput,
    Expected(String),
    ExpectedPairGotKey(String),
    DuplicateKey(String),
}

fn expect<T>(value: ParseOpt<T>, error_msg: &str) -> ParseResult<T> {
    match value? {
        Some(x) => Ok(x),
        None => Err(ParseError::Expected(error_msg.to_string())),
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
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
    fn expect_token(&mut self, expected: &str) -> ParseResult<()> {
        match (self.peek(), expected) {
            (Token::CloseParen(_), ")") => {
                self.advance();
            }
            (Token::OpenBrace(_), "{") => {
                self.advance();
            }
            (Token::CloseBrace(_), "}") => {
                self.advance();
            }
            (Token::ColonEquals(_), ":=") => {
                self.advance();
            }
            (Token::Colon(_), ":") => {
                self.advance();
            }
            (Token::CloseBracket(_), "]") => {
                self.advance();
            }
            (_, _) => return Err(ParseError::Expected(expected.to_string())),
        }
        Ok(())
    }
    fn key(&mut self) -> String {
        let mut parts = Vec::new();

        loop {
            match self.peek() {
                Token::Identifier(key, _) => {
                    parts.push(mem::take(key));
                    self.advance();
                }
                Token::Operator(key, _) => {
                    parts.push(mem::take(key));
                    self.advance();
                }
                Token::Integer(value, _) => {
                    parts.push(value.to_string());
                    self.advance();
                }
                _ => break,
            }
        }
        parts.join(" ")
    }

    fn param(&mut self, key: String, builder: &mut PairParamsBuilder) -> ParseResult<()> {
        let binding = expect(self.binding(), "binding")?;
        builder.add_value(key, binding)?;
        Ok(())
    }

    fn params(&mut self) -> ParseResult<ParamsBuilder> {
        let mut builder = PairParamsBuilder::new();
        loop {
            let key = self.key();
            if self.expect_token(":").is_ok() {
                self.param(key, &mut builder)?;
            } else if key.len() > 0 {
                return Ok(ParamsBuilder::key(key));
            } else {
                break;
            }
        }
        Ok(ParamsBuilder::PairBuilder(builder))
    }

    fn object(&mut self, src: Source) -> ParseResult<Expr> {
        let mut builder = ObjectBuilder::new();
        loop {
            match self.peek() {
                Token::On(_) => {
                    self.advance();
                    self.expect_token("{")?;
                    let params = self.params()?;
                    self.expect_token("}")?;
                    let body = self.body()?;
                    builder.add_on(params, body)?;
                }
                _ => break,
            }
        }
        Ok(Expr::Object(builder, src))
    }

    fn frame(&mut self, _: Source) -> ParseResult<Expr> {
        unimplemented!()
    }

    fn base_expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::Integer(value, source) => {
                let val = *value;
                let src = *source;
                self.advance();
                Ok(Some(Expr::Integer(val, src)))
            }
            Token::Float(value, source) => {
                let val = *value;
                let src = *source;
                self.advance();
                Ok(Some(Expr::Float(val, src)))
            }
            Token::String(value, source) => {
                let val = mem::take(value);
                let src = *source;
                self.advance();
                Ok(Some(Expr::String(val, src)))
            }
            Token::Identifier(value, source) => {
                let key = mem::take(value);
                let src = *source;
                self.advance();
                Ok(Some(Expr::Identifier(key, src)))
            }
            // TODO: parens create block, not just wrapping expr
            Token::OpenParen(source) => {
                let src = *source;
                self.advance();
                let body = self.body()?;
                self.expect_token(")")?;
                Ok(Some(Expr::Paren(body, src)))
            }
            Token::OpenBracket(source) => {
                let src = *source;
                self.advance();
                let expr = match self.peek() {
                    Token::On(_) => self.object(src)?,
                    _ => self.frame(src)?,
                };
                self.expect_token("]")?;
                Ok(Some(expr))
            }
            _ => Ok(None),
        }
    }

    fn arg(&mut self, builder: &mut SendBuilder, key: String) -> ParseResult<()> {
        let arg = expect(self.expr(), "expr")?;
        builder.add_value(key, arg)
    }

    fn call_expr_body(&mut self, target: Expr, source: Source) -> ParseOpt<Expr> {
        let mut builder = SendBuilder::new();
        loop {
            let key = self.key();
            if self.expect_token(":").is_ok() {
                self.arg(&mut builder, key)?;
            } else if key.len() > 0 {
                return Ok(Some(builder.build_key(key, target, source)?));
            } else {
                break;
            }
        }
        Ok(Some(builder.build(target, source)?))
    }

    fn call_expr(&mut self) -> ParseOpt<Expr> {
        let mut expr = match self.base_expr()? {
            Some(expr) => expr,
            None => return Ok(None),
        };
        while let Token::OpenBrace(src) = self.peek() {
            let source = *src;
            self.advance();
            expr = expect(self.call_expr_body(expr, source), "arg")?;
            self.expect_token("}")?;
        }
        Ok(Some(expr))
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
            _ => self.call_expr(),
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
                self.expect_token(":=")?;
                let expr = expect(self.expr(), "expr")?;

                Ok(Some(Stmt::Let(binding, expr)))
            }
            _ => match self.expr()? {
                Some(expr) => Ok(Some(Stmt::Expr(expr))),
                None => Ok(None),
            },
        }
    }
    pub fn body(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut out = Vec::new();
        while let Some(stmt) = self.stmt()? {
            out.push(stmt)
        }
        Ok(out)
    }
    pub fn program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let out = self.body()?;
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
        .is_ok());
    }

    #[test]
    fn sends() {
        assert!(parse("1{key: 2}").is_ok());
        assert!(parse("1{key}").is_ok());
        assert!(parse("1{long key 123}").is_ok());
        assert!(parse("1{: 1}").is_ok());
        assert!(parse("1{foo: 1 bar: 1}").is_ok());
        assert!(parse("1{foo: 1 foo: 1}").is_err());
    }

    #[test]
    fn objects() {
        assert!(parse("[on {x} 1]").is_ok());
        assert!(parse("[on {x: x} 1]").is_ok());
        assert!(parse("[on {x: x} 1]").is_ok());
        assert!(parse("[on {x: x y: y} x y]").is_ok());
    }
}
