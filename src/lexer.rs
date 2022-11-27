use crate::source::Source;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Comment(String, Source),
    Whitespace(Source),
    Integer(u64, Source),
    Identifier(String, Source),
    Let(Source),
    ColonEquals(Source),
    EndOfInput,
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
        let (start, ch) = self.chars.peek()?.to_owned();
        match ch {
            '#' => {
                self.chars.next();
                Some(self.comment(start))
            }
            ':' => {
                self.chars.next();
                if let Some((_, '=')) = self.chars.peek() {
                    self.chars.next();
                    return Some(Token::ColonEquals(Source::new(start, 2)));
                } else {
                    unimplemented!()
                }
            }
            '0'..='9' => {
                return Some(self.integer(start));
            }
            'a'..='z' | 'A'..='Z' => {
                return Some(self.ident_or_keyword(start));
            }
            ' ' | '\t' | '\n' => {
                return Some(self.whitespace(start));
            }
            _ => unimplemented!(),
        }
    }
    fn comment(&mut self, start: usize) -> Token {
        let mut str = String::new();
        while let Some((_, ch)) = self.chars.peek() {
            if *ch == '\n' {
                break;
            }
            str.push(*ch);
            self.chars.next();
        }
        let len = str.len();
        Token::Comment(str, Source::new(start, len))
    }
    fn whitespace(&mut self, start: usize) -> Token {
        let mut length = 0;
        while let Some((_, ch)) = self.chars.peek() {
            if !ch.is_whitespace() {
                break;
            }
            length += 1;
            self.chars.next();
        }
        Token::Whitespace(Source::new(start, length))
    }
    fn integer(&mut self, start: usize) -> Token {
        let mut value: u64 = 0;
        let mut length = 0;
        while let Some((_, ch)) = self.chars.peek() {
            if let Some(digit) = ch.to_digit(10) {
                value = (value * 10) + (digit as u64);
                length += 1;
                self.chars.next();
            } else if *ch == '_' {
                length += 1;
                self.chars.next();
            } else {
                break;
            }
        }
        Token::Integer(value, Source::new(start, length))
    }
    fn ident_or_keyword(&mut self, start: usize) -> Token {
        let mut str = String::new();
        while let Some((_, ch)) = self.chars.peek() {
            if ch.is_alphabetic() || ch.is_numeric() || *ch == '_' {
                str.push(*ch);
                self.chars.next();
            } else {
                break;
            }
        }
        let source = Source::new(start, str.len());
        match str.as_str() {
            "let" => Token::Let(source),
            _ => Token::Identifier(str, source),
        }
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
        lex("0");
        lex("23");
        lex("1_000");
    }

    #[test]
    fn numbers_comments_whitespace() {
        lex("");
        lex("# this is a comment");
        lex("123 456 # comment\n789");
    }
    #[test]
    fn let_identifiers() {
        lex("let foo_bar := 1
        let baz := foo_bar");
    }
}
