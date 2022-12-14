use std::collections::HashMap;

use crate::{
    compiler_2::{Binding, Expr, Object, Stmt},
    lexer_2::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Expected(String),
    ExpectedToken(Token),
    InvalidSendArgs,
    InvalidParams,
}

pub type Parse<T> = Result<T, ParseError>;
pub type ParseOpt<T> = Result<Option<T>, ParseError>;

fn expect<T>(name: &str, value: ParseOpt<T>) -> Parse<T> {
    match value {
        Ok(Some(value)) => Ok(value),
        _ => Err(ParseError::Expected(name.to_string())),
    }
}

struct ParamsBuilder {
    params: HashMap<String, Binding>,
}

impl ParamsBuilder {
    fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }
    fn key(self, key: String) -> Parse<(String, Vec<Binding>)> {
        if self.params.len() > 0 {
            if key.len() == 0 {
                return self.build();
            }
            return Err(ParseError::InvalidParams);
        }
        return Ok((key, vec![]));
    }
    fn add(&mut self, key: String, value: Binding) -> Parse<()> {
        if self.params.insert(key, value).is_some() {
            return Err(ParseError::InvalidParams);
        }
        return Ok(());
    }
    fn build(self) -> Parse<(String, Vec<Binding>)> {
        let mut selector = String::new();
        let mut params = Vec::new();

        let mut entries = self.params.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (key, param) in entries {
            selector.push_str(&key);
            selector.push(':');
            params.push(param);
        }

        Ok((selector, params))
    }
}

struct SendBuilder {
    target: Expr,
    args: HashMap<String, Expr>,
}

impl SendBuilder {
    fn new(target: Expr) -> Self {
        Self {
            target,
            args: HashMap::new(),
        }
    }
    fn key(self, key: String) -> Parse<Expr> {
        if self.args.len() > 0 {
            if key.len() == 0 {
                return self.build();
            }
            return Err(ParseError::InvalidSendArgs);
        }
        return Ok(Expr::Send(key, Box::new(self.target), vec![]));
    }
    fn add(&mut self, key: String, value: Expr) -> Parse<()> {
        if self.args.insert(key, value).is_some() {
            return Err(ParseError::InvalidSendArgs);
        }
        return Ok(());
    }
    fn build(self) -> Parse<Expr> {
        let mut selector = String::new();
        let mut args = Vec::new();

        let mut entries = self.args.into_iter().collect::<Vec<_>>();
        entries.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (key, arg) in entries {
            selector.push_str(&key);
            selector.push(':');
            args.push(arg);
        }

        Ok(Expr::Send(selector, Box::new(self.target), args))
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}
impl Parser {
    pub fn parse(tokens: Vec<Token>) -> Parse<Vec<Stmt>> {
        Self::new(tokens).program()
    }
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, index: 0 }
    }
    fn peek(&self) -> Token {
        self.tokens
            .get(self.index)
            .cloned()
            .unwrap_or(Token::EndOfInput)
    }
    fn advance(&mut self) {
        self.index += 1
    }

    fn expect_token(&mut self, token: Token) -> Parse<()> {
        if self.peek() == token {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::ExpectedToken(token))
        }
    }

    fn key(&mut self) -> Parse<String> {
        let mut parts = vec![];
        loop {
            match self.peek() {
                Token::Identifier(key) => {
                    self.advance();
                    parts.push(key);
                }
                Token::Operator(key) => {
                    self.advance();
                    parts.push(key);
                }
                Token::Do => {
                    self.advance();
                    parts.push("do".to_string());
                }
                Token::Let => {
                    self.advance();
                    parts.push("let".to_string());
                }
                Token::Set => {
                    self.advance();
                    parts.push("set".to_string());
                }
                Token::Var => {
                    self.advance();
                    parts.push("var".to_string());
                }
                Token::On => {
                    self.advance();
                    parts.push("on".to_string());
                }
                _ => return Ok(parts.join(" ")),
            }
        }
    }

    fn params_body(&mut self) -> Parse<(String, Vec<Binding>)> {
        let mut builder = ParamsBuilder::new();
        loop {
            let key = self.key()?;
            if self.expect_token(Token::Colon).is_ok() {
                let param = expect("param", self.binding())?;
                builder.add(key, param)?;
            } else {
                return builder.key(key);
            }
        }
    }

    fn handler(&mut self, object: &mut Object) -> Parse<()> {
        self.expect_token(Token::OpenBrace)?;
        let (selector, params) = self.params_body()?;
        self.expect_token(Token::CloseBrace)?;
        let body = self.body()?;
        object.add_handler(selector, params, body)?;
        Ok(())
    }

    fn base_expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::Integer(value) => {
                self.advance();
                Ok(Some(Expr::Integer(value)))
            }
            Token::Identifier(value) => {
                self.advance();
                Ok(Some(Expr::Identifier(value)))
            }
            Token::OpenBracket => {
                self.advance();
                let mut object = Object::new();
                loop {
                    match self.peek() {
                        Token::On => {
                            self.advance();
                            self.handler(&mut object)?;
                        }
                        _ => {
                            self.expect_token(Token::CloseBracket)?;
                            return Ok(Some(Expr::Object(object)));
                        }
                    }
                }
            }
            _ => Ok(None),
        }
    }

    fn send_body(&mut self, left: Expr) -> Parse<Expr> {
        let mut builder = SendBuilder::new(left);
        loop {
            let key = self.key()?;
            if self.expect_token(Token::Colon).is_ok() {
                let arg = expect("arg", self.expr())?;
                builder.add(key, arg)?;
            } else {
                return builder.key(key);
            }
        }
    }

    fn send_expr(&mut self) -> ParseOpt<Expr> {
        let mut left = if let Some(expr) = self.base_expr()? {
            expr
        } else {
            return Ok(None);
        };
        loop {
            match self.peek() {
                Token::OpenBrace => {
                    self.advance();
                    left = self.send_body(left)?;
                    self.expect_token(Token::CloseBrace)?;
                }
                _ => return Ok(Some(left)),
            }
        }
    }

    fn unary_op_expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::Operator(op) => {
                self.advance();
                let expr = expect("expr", self.send_expr())?;
                Ok(Some(Expr::Send(op, Box::new(expr), vec![])))
            }
            _ => self.send_expr(),
        }
    }

    fn expr(&mut self) -> ParseOpt<Expr> {
        let mut left = if let Some(expr) = self.unary_op_expr()? {
            expr
        } else {
            return Ok(None);
        };
        loop {
            match self.peek() {
                Token::Operator(op) => {
                    self.advance();
                    let expr = expect("expr", self.unary_op_expr())?;
                    left = Expr::Send(format!("{}:", op), Box::new(left), vec![expr]);
                }
                _ => return Ok(Some(left)),
            }
        }
    }

    fn binding(&mut self) -> ParseOpt<Binding> {
        match self.peek() {
            Token::Identifier(key) => {
                self.advance();
                Ok(Some(Binding::Identifier(key)))
            }
            _ => Ok(None),
        }
    }

    fn stmt(&mut self) -> ParseOpt<Stmt> {
        match self.peek() {
            Token::Let => {
                self.advance();
                let binding = expect("binding", self.binding())?;
                self.expect_token(Token::ColonEquals)?;
                let expr = expect("expr", self.expr())?;

                Ok(Some(Stmt::Let(binding, expr)))
            }
            _ => {
                if let Some(expr) = self.expr()? {
                    Ok(Some(Stmt::Expr(expr)))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn body(&mut self) -> Parse<Vec<Stmt>> {
        let mut out = vec![];
        while let Some(stmt) = self.stmt()? {
            out.push(stmt);
        }
        Ok(out)
    }

    fn program(&mut self) -> Parse<Vec<Stmt>> {
        let out = self.body()?;
        self.expect_token(Token::EndOfInput)?;
        Ok(out)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Token::*;

    fn assert_ok(code: Vec<Token>, expected: Vec<Stmt>) {
        assert_eq!(Parser::parse(code), Ok(expected))
    }

    fn assert_err(code: Vec<Token>, expected: ParseError) {
        assert_eq!(Parser::parse(code), Err(expected))
    }

    fn ident(str: &str) -> Token {
        Token::Identifier(str.to_string())
    }

    #[test]
    fn empty_program() {
        assert_ok(vec![], vec![]);
    }

    #[test]
    fn number_literals() {
        assert_ok(
            vec![Integer(123), Integer(456)],
            vec![
                Stmt::Expr(Expr::Integer(123)),
                Stmt::Expr(Expr::Integer(456)),
            ],
        )
    }

    #[test]
    fn assignment() {
        assert_ok(
            vec![Let, ident("x"), ColonEquals, Integer(123)],
            vec![Stmt::Let(
                Binding::Identifier("x".to_string()),
                Expr::Integer(123),
            )],
        )
    }

    #[test]
    fn unexpected_end_of_input() {
        assert_err(
            vec![Let, ident("x"), ColonEquals],
            ParseError::Expected("expr".to_string()),
        )
    }

    #[test]
    fn expected_end_of_input() {
        assert_err(vec![OpenParen], ParseError::ExpectedToken(EndOfInput))
    }
}
