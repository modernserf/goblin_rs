use crate::grammar::Token;

pub struct Lexer {
    chars: Vec<char>,
    index: usize,
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
            '"' => self.string(),
            '_' => self.quoted_identifier(),
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
            '?' => self.accept(Token::QuestionMark),
            '\0' => Token::EndOfInput,
            ch => {
                if Token::is_operator(ch) {
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
                return Token::from_ident(str);
            }
        }
    }
    fn quoted_identifier(&mut self) -> Token {
        let mut str = String::new();
        self.advance();
        loop {
            let ch = self.peek();
            if ch == '_' {
                self.advance();
                return Token::QuotedIdentifier(str);
            } else {
                self.advance();
                str.push(ch);
            }
        }
    }
    fn operator(&mut self) -> Token {
        let mut str = String::new();
        loop {
            let ch = self.peek();
            if Token::is_operator(ch) {
                self.advance();
                str.push(ch);
            } else {
                return Token::Operator(str);
            }
        }
    }
    fn string(&mut self) -> Token {
        let mut str = String::new();
        self.advance();
        loop {
            let ch = self.peek();
            if ch == '"' {
                self.advance();
                return Token::String(str);
            } else {
                self.advance();
                str.push(ch);
            }
        }
    }
}
