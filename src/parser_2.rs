use crate::{
    compiler_2::{Binding, Expr, Stmt},
    lexer_2::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Expected(String),
    ExpectedToken(Token),
}

type Parse<T> = Result<T, ParseError>;
type ParseOpt<T> = Result<Option<T>, ParseError>;

fn expect<T>(name: &str, value: ParseOpt<T>) -> Parse<T> {
    match value {
        Ok(Some(value)) => Ok(value),
        _ => Err(ParseError::Expected(name.to_string())),
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

    fn expr(&mut self) -> ParseOpt<Expr> {
        match self.peek() {
            Token::Integer(value) => {
                self.advance();
                Ok(Some(Expr::Integer(value)))
            }
            Token::Identifier(value) => {
                self.advance();
                Ok(Some(Expr::Identifier(value)))
            }
            _ => Ok(None),
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

    fn program(&mut self) -> Parse<Vec<Stmt>> {
        let mut out = vec![];
        while let Some(stmt) = self.stmt()? {
            out.push(stmt);
        }
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
