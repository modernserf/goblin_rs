use crate::frame::FrameBuilder;
use crate::object_builder::{ObjectBuilder, PairParamsBuilder, ParamsBuilder};
use crate::send_builder::SendBuilder;
use crate::{
    lexer::{Lexer, Token},
    parse_binding::Binding,
    parse_expr::Expr,
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
    DuplicateHandler(String),
    DuplicateElseHandler,
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
            (Token::Then(_), "then") => {
                self.advance();
            }
            (Token::End(_), "end") => {
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

    fn ident(&mut self) -> ParseResult<String> {
        match self.peek() {
            Token::Identifier(value, _) => {
                let key = mem::take(value);
                self.advance();
                Ok(key)
            }
            Token::QuotedIdent(value, _) => {
                let key = mem::take(value);
                self.advance();
                Ok(key)
            }
            _ => Err(ParseError::Expected("identifier".to_string())),
        }
    }

    fn param(&mut self, key: String, builder: &mut PairParamsBuilder) -> ParseResult<()> {
        match self.peek() {
            Token::Do(_) => {
                self.advance();
                let ident = self.ident()?;
                builder.add_do(key, ident)?;
            }
            Token::Var(_) => {
                self.advance();
                let ident = self.ident()?;
                builder.add_var(key, ident)?;
            }
            _ => {
                let binding = expect(self.binding(), "binding")?;
                builder.add_value(key, binding)?;
            }
        }
        Ok(())
    }

    fn params(&mut self) -> ParseResult<ParamsBuilder> {
        let mut builder = PairParamsBuilder::new();
        loop {
            if let Token::QuotedIdent(key, src) = self.peek() {
                let src = *src;
                let key = mem::take(key);
                self.advance();
                let binding = Binding::Identifier(key.clone(), src);
                builder.add_value(key, binding)?;
                continue;
            }

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

    fn object(&mut self) -> ParseResult<ObjectBuilder> {
        let mut builder = ObjectBuilder::new();
        match self.peek() {
            Token::OpenBrace(_) => {
                self.advance();
                let params = self.params()?;
                self.expect_token("}")?;
                let body = self.body()?;
                let _ = self.expect_token("end");
                builder.add_on(params, body)?;
                return Ok(builder);
            }
            _ => {}
        }

        loop {
            match self.peek() {
                Token::On(_) => {
                    self.advance();
                    self.expect_token("{")?;
                    let params = self.params()?;
                    self.expect_token("}")?;
                    let body = self.body()?;
                    let _ = self.expect_token("end");
                    builder.add_on(params, body)?;
                }
                Token::Else(_) => {
                    self.advance();
                    let body = self.body()?;
                    let _ = self.expect_token("end");
                    builder.add_else(body)?;
                }
                _ => break,
            }
        }
        Ok(builder)
    }

    fn frame(&mut self, source: Source) -> ParseResult<Expr> {
        let mut builder = FrameBuilder::new();
        loop {
            if let Token::QuotedIdent(key, source) = self.peek() {
                let key = mem::take(key);
                let source = *source;
                self.advance();
                let value = Expr::Identifier(key.clone(), source);
                builder.add_pair(key, value)?;
                continue;
            }
            let key = self.key();
            if self.expect_token(":").is_ok() {
                let value = expect(self.expr(), "expr")?;
                builder.add_pair(key, value)?;
            } else if key.len() > 0 {
                return builder.build_key(key, source);
            } else {
                break;
            }
        }
        builder.build(source)
    }

    fn if_expr(&mut self, source: Source) -> ParseResult<Expr> {
        let cond = expect(self.expr(), "expr")?;
        self.expect_token("then")?;
        let if_true = self.body()?;
        match self.peek() {
            Token::End(_) => {
                self.advance();
                Ok(Expr::If(Box::new(cond), if_true, vec![], source))
            }
            Token::Else(_) => {
                self.advance();
                if let Token::If(next_source) = self.peek() {
                    let next_source = *next_source;
                    self.advance();
                    let res = self.if_expr(next_source)?;
                    return Ok(Expr::If(
                        Box::new(cond),
                        if_true,
                        vec![Stmt::Expr(res)],
                        source,
                    ));
                }
                let if_false = self.body()?;
                self.expect_token("end")?;
                Ok(Expr::If(Box::new(cond), if_true, if_false, source))
            }
            _ => Err(ParseError::Expected("else / end".to_string())),
        }
    }

    fn base_expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::SelfRef(source) => {
                let src = *source;
                self.advance();
                Ok(Some(Expr::SelfRef(src)))
            }
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
            Token::QuotedIdent(value, source) => {
                let key = mem::take(value);
                let src = *source;
                self.advance();
                Ok(Some(Expr::Identifier(key, src)))
            }
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
                    Token::OpenBrace(_) => {
                        let builder = self.object()?;
                        Expr::Object(builder, src)
                    }
                    Token::On(_) => {
                        let builder = self.object()?;
                        Expr::Object(builder, src)
                    }
                    _ => self.frame(src)?,
                };
                self.expect_token("]")?;
                Ok(Some(expr))
            }
            Token::If(source) => {
                let source = *source;
                self.advance();
                Ok(Some(self.if_expr(source)?))
            }
            _ => Ok(None),
        }
    }

    fn arg(&mut self, builder: &mut SendBuilder, key: String) -> ParseResult<()> {
        match self.peek() {
            Token::On(_) => {
                let obj = self.object()?;
                builder.add_do(key, obj)?;
            }
            Token::OpenBrace(_) => {
                let obj = self.object()?;
                builder.add_do(key, obj)?;
            }
            Token::Var(_) => {
                self.advance();
                let ident = self.ident()?;
                builder.add_var(key, ident)?;
            }
            _ => {
                let arg = expect(self.expr(), "expr")?;
                builder.add_value(key, arg)?;
            }
        }
        Ok(())
    }

    fn call_expr_body(&mut self, target: Expr, source: Source) -> ParseOpt<Expr> {
        let mut builder = SendBuilder::new();
        loop {
            if let Token::QuotedIdent(key, source) = self.peek() {
                let key = mem::take(key);
                let source = *source;
                self.advance();
                let value = Expr::Identifier(key.clone(), source);
                builder.add_value(key, value)?;
                continue;
            }

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
                let operator = mem::take(value);
                self.advance();
                let target = expect(self.unary_op_expr(), "expr")?;
                let res = SendBuilder::unary_op(operator, target, src)?;
                Ok(Some(res))
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
            let mut operator = mem::take(value);
            operator.push(':');
            self.advance();
            let operand = expect(self.unary_op_expr(), "expr")?;
            expr = SendBuilder::binary_op(operator, expr, operand, src)?;
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
            Token::QuotedIdent(value, source) => {
                let key = mem::take(value);
                let src = *source;
                self.advance();
                if key == "" {
                    Ok(Some(Binding::Placeholder(src)))
                } else {
                    Ok(Some(Binding::Identifier(key, src)))
                }
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
            Token::Var(_) => {
                self.advance();
                let binding = expect(self.binding(), "binding")?;
                self.expect_token(":=")?;
                let expr = expect(self.expr(), "expr")?;
                Ok(Some(Stmt::Var(binding, expr)))
            }
            Token::Set(_) => {
                self.advance();
                let target = expect(self.expr(), "set target")?;
                if self.expect_token(":=").is_ok() {
                    let binding = target.as_binding()?;
                    let expr = expect(self.expr(), "expr")?;
                    Ok(Some(Stmt::Set(binding, expr)))
                } else {
                    let set_in_place = target.as_set_in_place()?;
                    Ok(Some(set_in_place))
                }
            }
            Token::Return(_) => {
                self.advance();
                let opt_expr = self.expr()?;
                Ok(Some(Stmt::Return(opt_expr)))
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

    fn assert_ok(str: &str) {
        match parse(str) {
            Ok(_) => {}
            Err(err) => panic!("{:?}", err),
        }
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

    #[test]
    fn pun_params() {
        assert_ok("[on {_x_ _y_} x + y]");
    }
}
