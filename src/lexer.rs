use std::collections::HashSet;

use crate::source::Source;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Comment(String, Source),
    Whitespace(Source),
    Integer(u64, Source),
    Float(f64, Source),
    String(String, Source),
    Identifier(String, Source),
    QuotedIdent(String, Source),
    Operator(String, Source),
    On(Source),
    Let(Source),
    Var(Source),
    Set(Source),
    Do(Source),
    If(Source),
    Then(Source),
    Else(Source),
    End(Source),
    Return(Source),
    SelfRef(Source),
    Colon(Source),
    ColonEquals(Source),
    OpenParen(Source),
    CloseParen(Source),
    OpenBrace(Source),
    CloseBrace(Source),
    OpenBracket(Source),
    CloseBracket(Source),
    QuestionMark(Source),
    EndOfInput,
}

type CharIter<'a> = std::iter::Peekable<std::iter::Enumerate<std::str::Chars<'a>>>;

pub struct Lexer<'a> {
    chars: CharIter<'a>,
    operators: HashSet<char>,
}

impl<'a> Lexer<'a> {
    pub fn from_string(string: &str) -> Lexer {
        let chars = string.chars().enumerate().peekable();
        Lexer {
            chars,
            operators: HashSet::from_iter("~!@$%^&*-+=|/,<>".chars()),
        }
    }
    fn get_token(&mut self) -> Token {
        let (start, ch) = match self.chars.peek() {
            Some(p) => p.to_owned(),
            None => return Token::EndOfInput,
        };
        match ch {
            '#' => {
                self.chars.next();
                self.comment(start)
            }
            '(' => {
                self.chars.next();
                return Token::OpenParen(Source::new(start, 1));
            }
            ')' => {
                self.chars.next();
                return Token::CloseParen(Source::new(start, 1));
            }
            '{' => {
                self.chars.next();
                return Token::OpenBrace(Source::new(start, 1));
            }
            '}' => {
                self.chars.next();
                return Token::CloseBrace(Source::new(start, 1));
            }
            '[' => {
                self.chars.next();
                return Token::OpenBracket(Source::new(start, 1));
            }
            ']' => {
                self.chars.next();
                return Token::CloseBracket(Source::new(start, 1));
            }
            '?' => {
                self.chars.next();
                return Token::QuestionMark(Source::new(start, 1));
            }
            ':' => {
                self.chars.next();
                if let Some((_, '=')) = self.chars.peek() {
                    self.chars.next();
                    return Token::ColonEquals(Source::new(start, 2));
                } else {
                    return Token::Colon(Source::new(start, 1));
                }
            }
            '"' => {
                self.chars.next();
                return self.string(start);
            }
            '_' => {
                self.chars.next();
                return self.quoted_ident(start);
            }
            '0'..='9' => {
                return self.number(start);
            }
            'a'..='z' | 'A'..='Z' => {
                return self.ident_or_keyword(start);
            }
            ' ' | '\t' | '\n' => {
                return self.whitespace(start);
            }
            _ => {
                if self.operators.contains(&ch) {
                    return self.operator(start);
                } else {
                    unimplemented!()
                }
            }
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
    fn number(&mut self, start: usize) -> Token {
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
            } else if *ch == '.' {
                length += 1;
                self.chars.next();
                return self.float(start, length, value as f64);
            } else {
                break;
            }
        }
        Token::Integer(value, Source::new(start, length))
    }
    fn float(&mut self, start: usize, mut length: usize, mut value: f64) -> Token {
        let mut mult = 0.1;
        while let Some((_, ch)) = self.chars.peek() {
            if let Some(digit) = ch.to_digit(10) {
                value += digit as f64 * mult;
                mult *= 0.1;
                length += 1;
                self.chars.next();
            } else if *ch == '_' {
                length += 1;
                self.chars.next();
            } else {
                break;
            }
        }
        Token::Float(value, Source::new(start, length))
    }
    fn string(&mut self, start: usize) -> Token {
        let mut len = 1;
        let mut str = String::new();
        while let Some((_, ch)) = self.chars.peek() {
            if *ch == '"' {
                len += 1;
                self.chars.next();
                return Token::String(str, Source::new(start, len));
            } else if *ch == '\\' {
                len += 1;
                self.chars.next();
                if let Some((_, ch)) = self.chars.next() {
                    len += 1;
                    str.push(ch);
                } else {
                    unimplemented!()
                }
            } else {
                str.push(*ch);
                len += 1;
                self.chars.next();
            }
        }
        unimplemented!()
    }
    fn ident_or_keyword(&mut self, start: usize) -> Token {
        let mut str = String::new();
        while let Some((_, ch)) = self.chars.peek() {
            if ch.is_alphabetic() || ch.is_numeric() || *ch == '_' || *ch == '\'' {
                str.push(*ch);
                self.chars.next();
            } else {
                break;
            }
        }
        let source = Source::new(start, str.len());
        match str.as_str() {
            "let" => Token::Let(source),
            "var" => Token::Var(source),
            "set" => Token::Set(source),
            "on" => Token::On(source),
            "self" => Token::SelfRef(source),
            "do" => Token::Do(source),
            "if" => Token::If(source),
            "then" => Token::Then(source),
            "else" => Token::Else(source),
            "end" => Token::End(source),
            "return" => Token::Return(source),
            _ => Token::Identifier(str, source),
        }
    }
    fn quoted_ident(&mut self, start: usize) -> Token {
        let mut len = 1;
        let mut str = String::new();
        while let Some((_, ch)) = self.chars.peek() {
            if *ch == '_' {
                len += 1;
                self.chars.next();
                return Token::QuotedIdent(str, Source::new(start, len));
            } else if *ch == '\\' {
                len += 1;
                self.chars.next();
                if let Some((_, ch)) = self.chars.next() {
                    len += 1;
                    str.push(ch);
                } else {
                    unimplemented!()
                }
            } else {
                str.push(*ch);
                len += 1;
                self.chars.next();
            }
        }
        unimplemented!()
    }
    fn operator(&mut self, start: usize) -> Token {
        let mut str = String::new();
        while let Some((_, ch)) = self.chars.peek() {
            if !self.operators.contains(ch) {
                break;
            }
            str.push(*ch);
            self.chars.next();
        }
        let source = Source::new(start, str.len());
        Token::Operator(str, source)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.get_token() {
                // drop non-semantic tokens
                Token::Whitespace(_) => {}
                Token::Comment(_, _) => {}
                // stop at End of Input
                Token::EndOfInput => return None,
                // produce value
                tok => return Some(tok),
            }
        }
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
    fn test_file() {
        let file = include_str!("test.gob");
        lex(file);
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
    #[test]
    fn operators() {
        lex("+ --> !@");
    }
    #[test]
    fn sends() {
        lex("1{key}");
    }
}
