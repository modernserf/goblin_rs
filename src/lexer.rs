#[derive(Debug, PartialEq, Clone)]
pub enum TokenValue {
    Integer(u64),
    EndOfInput,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub value: TokenValue,
    pub start: usize,
    pub length: usize,
}

impl Token {
    pub fn integer(value: u64, start: usize, length: usize) -> Self {
        Token {
            value: TokenValue::Integer(value),
            start,
            length,
        }
    }
    pub fn eof() -> Self {
        Token {
            value: TokenValue::EndOfInput,
            start: 0,
            length: 0,
        }
    }
}

type CharIter<'a> = std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'a>>>;

pub struct Lexer<'a> {
    chars: CharIter<'a>,
}

impl<'a> Lexer<'a> {
    pub fn from_string(string: &str) -> Lexer {
        let chars = string.chars().enumerate().peekable();
        Lexer { chars }
    }
    fn get_token(&mut self) -> Option<Token> {
        let (_, ch) = self.chars.peek()?;
        match ch {
            // TODO: match specific chars
            _ => {
                if ch.is_numeric() {
                    return self.integer();
                } else {
                    unimplemented!();
                }
            }
        }
    }
    fn integer(&mut self) -> Option<Token> {
        let start = self.chars.peek()?.0;
        let mut value: u64 = 0;
        let mut length = 0;
        loop {
            if let Some((_, ch)) = self.chars.peek() {
                if let Some(digit) = ch.to_digit(10) {
                    value = (value * 10) + (digit as u64);
                    length += 1;
                    self.chars.next();
                    continue;
                } else if *ch == '_' {
                    self.chars.next();
                    length += 1;
                    continue;
                }
            }
            break;
        }

        Some(Token::integer(value, start, length))
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.get_token()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn lex(string: &str) -> Vec<Token> {
        Lexer::from_string(&string)
            .into_iter()
            .collect::<Vec<Token>>()
    }

    #[test]
    fn numbers() {
        assert_eq!(lex("0"), vec![Token::integer(0, 0, 1)]);
        assert_eq!(lex("23"), vec![Token::integer(23, 0, 2)]);
        assert_eq!(lex("1_000"), vec![Token::integer(1000, 0, 5)]);
    }
}
