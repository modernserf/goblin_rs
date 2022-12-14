use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i64),
    Identifier(String),
    Operator(String),
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Colon,
    ColonEquals,
    Let,
    Var,
    Do,
    Set,
    On,
    EndOfInput,
}

pub struct Lexer {
    chars: Vec<char>,
    index: usize,
}

thread_local! {
  static OPERATORS: HashSet<char> = HashSet::from_iter("~!@$%^&*-+=|/,<>".chars());
}

fn is_operator(ch: char) -> bool {
    OPERATORS.with(|set| set.contains(&ch))
}

impl Lexer {
    pub fn lex(str: String) -> Vec<Token> {
        let mut out = vec![];
        let mut lexer = Lexer::new(str);
        loop {
            let tok = lexer.next();
            if tok == Token::EndOfInput {
                return out;
            }
            out.push(tok);
        }
    }
    fn new(str: String) -> Self {
        Lexer {
            chars: str.chars().collect(),
            index: 0,
        }
    }
    fn peek(&self) -> char {
        if self.index >= self.chars.len() {
            return '\0';
        }
        self.chars[self.index]
    }
    fn advance(&mut self) {
        self.index += 1;
    }
    fn next(&mut self) -> Token {
        match self.peek() {
            // these produce no tokens but call next recursively
            '#' => self.comment(),
            ' ' | '\t' | '\n' => self.whitespace(),
            // actually produce values
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' => self.identifier_or_keyword(),
            ':' => {
                self.advance();
                match self.peek() {
                    '=' => self.accept(Token::ColonEquals),
                    _ => Token::Colon,
                }
            }
            '(' => self.accept(Token::OpenParen),
            ')' => self.accept(Token::CloseParen),
            '{' => self.accept(Token::OpenBrace),
            '}' => self.accept(Token::CloseBrace),
            '[' => self.accept(Token::OpenBracket),
            ']' => self.accept(Token::CloseBracket),
            '\0' => Token::EndOfInput,
            ch => {
                if is_operator(ch) {
                    self.operator()
                } else {
                    panic!("unknown char")
                }
            }
        }
    }

    fn accept(&mut self, token: Token) -> Token {
        self.advance();
        token
    }

    fn comment(&mut self) -> Token {
        loop {
            match self.peek() {
                '\n' | '\0' => return self.next(),
                _ => {
                    self.advance();
                }
            }
        }
    }
    fn whitespace(&mut self) -> Token {
        loop {
            if self.peek().is_whitespace() {
                self.advance()
            } else {
                return self.next();
            }
        }
    }
    fn number(&mut self) -> Token {
        let mut sum = 0;
        while let Some(digit) = self.peek().to_digit(10) {
            self.advance();
            sum = sum * 10 + (digit as i64)
        }
        Token::Integer(sum)
    }
    fn identifier_or_keyword(&mut self) -> Token {
        let mut str = String::new();
        loop {
            let ch = self.peek();
            if ch.is_alphanumeric() || ch == '\'' || ch == '_' {
                self.advance();
                str.push(ch);
            } else {
                return match str.as_str() {
                    "let" => Token::Let,
                    "var" => Token::Var,
                    "do" => Token::Do,
                    "set" => Token::Set,
                    "on" => Token::On,
                    _ => Token::Identifier(str),
                };
            }
        }
    }
    fn operator(&mut self) -> Token {
        let mut str = String::new();
        loop {
            let ch = self.peek();
            if is_operator(ch) {
                self.advance();
                str.push(ch);
            } else {
                return Token::Operator(str);
            }
        }
    }
}
