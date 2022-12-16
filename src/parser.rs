use std::collections::HashMap;

use crate::{
    ast::{Binding, Expr, Object, Stmt},
    lexer::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Expected(String),
    ExpectedToken(Token),
    DuplicateKey(String),
    MixedKeyPair(String),
}

impl ParseError {
    pub fn expected(str: &str) -> Self {
        Self::Expected(str.to_string())
    }
}

pub type Parse<T> = Result<T, ParseError>;
pub type ParseOpt<T> = Result<Option<T>, ParseError>;

fn expect<T>(name: &str, value: ParseOpt<T>) -> Parse<T> {
    match value {
        Ok(Some(value)) => Ok(value),
        _ => Err(ParseError::expected(name)),
    }
}

struct SelectorBuilderResult<T> {
    selector: String,
    items: Vec<(String, T)>,
}

struct SelectorBuilder<T> {
    items: HashMap<String, T>,
}

impl<T> SelectorBuilder<T> {
    fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
    fn add(&mut self, key: String, value: T) -> Parse<()> {
        if self.items.contains_key(&key) {
            return Err(ParseError::DuplicateKey(key));
        }

        self.items.insert(key, value);
        return Ok(());
    }
    fn resolve(self, last_key: String) -> Parse<SelectorBuilderResult<T>> {
        if self.items.len() > 0 {
            if last_key.len() == 0 {
                return self.resolve_pairs();
            }
            return Err(ParseError::MixedKeyPair(last_key));
        }
        return Ok(SelectorBuilderResult {
            selector: last_key,
            items: vec![],
        });
    }
    fn resolve_pairs(self) -> Parse<SelectorBuilderResult<T>> {
        let mut items = self.items.into_iter().collect::<Vec<_>>();
        items.sort_by(|(a, _), (b, _)| a.cmp(b));
        let selector = items
            .iter()
            .map(|p| &p.0)
            .fold(String::new(), |l, r| format!("{}{}:", l, r));
        Ok(SelectorBuilderResult { selector, items })
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
                Token::Integer(num) => {
                    self.advance();
                    parts.push(num.to_string());
                }
                tok => {
                    if let Some(str) = tok.to_keyword() {
                        self.advance();
                        parts.push(str)
                    } else {
                        return Ok(parts.join(" "));
                    }
                }
            }
        }
    }

    fn handler(&mut self, object: &mut Object) -> Parse<()> {
        let mut heads = vec![];
        while self.expect_token(Token::OpenBrace).is_ok() {
            let result = self.build_structure(|p| p.param())?;
            self.expect_token(Token::CloseBrace)?;
            heads.push(result);
        }
        if heads.is_empty() {
            return Err(ParseError::expected("params"));
        }
        let body = self.body()?;

        for head in heads {
            let params = head.items.into_iter().map(|p| p.1).collect();
            object.add_handler(head.selector, params, body.clone())?;
        }
        Ok(())
    }

    fn object_body(&mut self) -> Parse<Object> {
        let mut object = Object::new();
        // single handler shorthand
        if let Token::OpenBrace = self.peek() {
            // brace consumed by handler
            self.handler(&mut object)?;
            return Ok(object);
        }

        loop {
            match self.peek() {
                Token::On => {
                    self.advance();
                    self.handler(&mut object)?;
                }
                _ => {
                    return Ok(object);
                }
            }
        }
    }

    fn build_structure<T>(
        &mut self,
        parse_item: fn(&mut Parser) -> Parse<T>,
    ) -> Parse<SelectorBuilderResult<T>> {
        let mut builder = SelectorBuilder::new();
        loop {
            match self.peek() {
                Token::QuotedIdentifier(key) => {
                    // token is read twice: first as key, then as value
                    let item = parse_item(self)?;
                    builder.add(key, item)?;
                }
                _ => {
                    let key = self.key()?;
                    if self.expect_token(Token::Colon).is_ok() {
                        let item = parse_item(self)?;
                        builder.add(key, item)?;
                    } else {
                        return builder.resolve(key);
                    }
                }
            }
        }
    }

    fn base_expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::SelfRef => {
                self.advance();
                Ok(Some(Expr::SelfRef))
            }
            Token::True => {
                self.advance();
                Ok(Some(Expr::Bool(true)))
            }
            Token::False => {
                self.advance();
                Ok(Some(Expr::Bool(false)))
            }
            Token::Integer(value) => {
                self.advance();
                Ok(Some(Expr::Integer(value)))
            }
            Token::String(str) => {
                self.advance();
                Ok(Some(Expr::String(str)))
            }
            Token::Identifier(value) => {
                self.advance();
                Ok(Some(Expr::Identifier(value)))
            }
            Token::QuotedIdentifier(value) => {
                self.advance();
                Ok(Some(Expr::Identifier(value)))
            }
            Token::OpenBracket => {
                self.advance();
                match self.peek() {
                    Token::On | Token::OpenBrace => {
                        let object = self.object_body()?;
                        self.expect_token(Token::CloseBracket)?;
                        return Ok(Some(Expr::Object(object)));
                    }
                    _ => {
                        let frame = self.build_structure(|p| p.arg())?;
                        self.expect_token(Token::CloseBracket)?;
                        return Ok(Some(Expr::Frame(frame.selector, frame.items)));
                    }
                }
            }
            Token::If => {
                self.advance();
                let cond = expect("expr", self.expr())?;
                self.expect_token(Token::Then)?;
                let if_true = self.body()?;
                match self.peek() {
                    Token::End => {
                        self.advance();
                        Ok(Some(Expr::If(Box::new(cond), if_true, vec![])))
                    }
                    Token::Else => {
                        self.advance();
                        match self.peek() {
                            Token::If => {
                                // token consumed in recursion
                                let next = expect("expr", self.expr())?;
                                Ok(Some(Expr::If(
                                    Box::new(cond),
                                    if_true,
                                    vec![Stmt::Expr(next)],
                                )))
                            }
                            _ => {
                                let if_false = self.body()?;
                                self.expect_token(Token::End)?;
                                Ok(Some(Expr::If(Box::new(cond), if_true, if_false)))
                            }
                        }
                    }
                    _ => Err(ParseError::expected("else | end")),
                }
            }
            Token::OpenParen => {
                self.advance();
                let body = self.body()?;
                self.expect_token(Token::CloseParen)?;
                Ok(Some(Expr::Paren(body)))
            }
            _ => Ok(None),
        }
    }

    fn send_expr(&mut self) -> ParseOpt<Expr> {
        if let Some(mut left) = self.base_expr()? {
            loop {
                match self.peek() {
                    Token::OpenBrace => {
                        self.advance();
                        let result = self.build_structure(|p| p.arg())?;
                        let args = result.items.into_iter().map(|p| p.1).collect();
                        left = Expr::Send(result.selector, Box::new(left), args);

                        self.expect_token(Token::CloseBrace)?;
                    }
                    _ => return Ok(Some(left)),
                }
            }
        }
        Ok(None)
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
        if let Some(mut left) = self.unary_op_expr()? {
            loop {
                match self.peek() {
                    Token::Operator(op) => {
                        self.advance();
                        let expr = expect("expr", self.unary_op_expr())?;
                        left = Expr::Send(format!("{}:", op), Box::new(left), vec![expr]);
                    }
                    Token::QuestionMark => {
                        self.advance();
                        if let Expr::Send(selector, target, args) = left {
                            let or_else = expect("expr", self.unary_op_expr())?;
                            left = Expr::TrySend(selector, target, args, Box::new(or_else));
                        } else {
                            return Err(ParseError::expected("try send"));
                        }
                    }
                    _ => return Ok(Some(left)),
                }
            }
        }
        Ok(None)
    }

    fn ident(&mut self) -> ParseOpt<String> {
        match self.peek() {
            Token::Identifier(key) => {
                self.advance();
                Ok(Some(key))
            }
            Token::QuotedIdentifier(key) => {
                self.advance();
                Ok(Some(key))
            }
            _ => Ok(None),
        }
    }

    fn arg(&mut self) -> Parse<Expr> {
        match self.peek() {
            Token::Var => {
                self.advance();
                if let Some(key) = self.ident()? {
                    return Ok(Expr::VarArg(key));
                }
                return Err(ParseError::expected("var"));
            }
            Token::On | Token::OpenBrace => {
                // object_body accepts On tokens
                let object = self.object_body()?;
                return Ok(Expr::DoArg(object));
            }
            _ => expect("arg", self.expr()),
        }
    }

    fn binding(&mut self) -> Parse<Binding> {
        match self.peek() {
            Token::Identifier(key) => {
                self.advance();
                Ok(Binding::Identifier(key))
            }
            Token::QuotedIdentifier(key) => {
                self.advance();
                Ok(Binding::Identifier(key))
            }
            Token::OpenBracket => {
                self.advance();
                let result = self.build_structure(|p| p.binding())?;
                self.expect_token(Token::CloseBracket)?;
                Ok(Binding::Destructure(result.items))
            }
            _ => Err(ParseError::expected("binding")),
        }
    }

    fn param(&mut self) -> Parse<Binding> {
        match self.peek() {
            Token::Var => {
                self.advance();
                if let Some(key) = self.ident()? {
                    return Ok(Binding::VarIdentifier(key));
                }
                return Err(ParseError::expected("var param"));
            }
            Token::Do => {
                self.advance();
                if let Some(key) = self.ident()? {
                    return Ok(Binding::DoIdentifier(key));
                }
                return Err(ParseError::expected("do param"));
            }
            _ => self.binding(),
        }
    }

    fn import_source(&mut self) -> Parse<String> {
        if let Token::String(str) = self.peek() {
            self.advance();
            Ok(str)
        } else {
            Err(ParseError::expected("import source"))
        }
    }

    fn stmt(&mut self) -> ParseOpt<Stmt> {
        match self.peek() {
            Token::Let => {
                self.advance();
                let binding = self.binding()?;
                self.expect_token(Token::ColonEquals)?;
                let expr = expect("expr", self.expr())?;

                Ok(Some(Stmt::Let(binding, expr, false)))
            }
            Token::Var => {
                self.advance();
                let binding = self.binding()?;
                self.expect_token(Token::ColonEquals)?;
                let expr = expect("expr", self.expr())?;

                Ok(Some(Stmt::Var(binding, expr)))
            }
            Token::Set => {
                self.advance();
                let target = expect("set target", self.expr())?;
                if self.expect_token(Token::ColonEquals).is_ok() {
                    let binding = target.as_binding()?;
                    let expr = expect("expr", self.expr())?;
                    Ok(Some(Stmt::Set(binding, expr)))
                } else {
                    let binding = target.set_target()?;
                    Ok(Some(Stmt::Set(binding, target)))
                }
            }
            Token::Import => {
                self.advance();
                let binding = self.binding()?;
                self.expect_token(Token::ColonEquals)?;
                let source = self.import_source()?;
                Ok(Some(Stmt::Import(binding, source, false)))
            }
            Token::Export => {
                self.advance();
                match self.peek() {
                    Token::Let => {
                        self.advance();
                        let binding = self.binding()?;
                        self.expect_token(Token::ColonEquals)?;
                        let expr = expect("expr", self.expr())?;

                        Ok(Some(Stmt::Let(binding, expr, true)))
                    }
                    Token::Import => {
                        self.advance();
                        let binding = self.binding()?;
                        self.expect_token(Token::ColonEquals)?;
                        let source = self.import_source()?;
                        Ok(Some(Stmt::Import(binding, source, true)))
                    }
                    _ => Err(ParseError::expected("export")),
                }
            }
            Token::Return => {
                self.advance();
                if let Some(expr) = self.expr()? {
                    Ok(Some(Stmt::Return(expr)))
                } else {
                    Ok(Some(Stmt::Return(Expr::Unit)))
                }
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
                false,
            )],
        )
    }

    #[test]
    fn unexpected_end_of_input() {
        assert_err(
            vec![Let, ident("x"), ColonEquals],
            ParseError::expected("expr"),
        )
    }

    #[test]
    fn expected_end_of_input() {
        assert_err(vec![CloseParen], ParseError::ExpectedToken(EndOfInput))
    }
}
