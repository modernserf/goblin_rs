use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Integer(i64),
    Identifier(String),
    QuotedIdentifier(String),
    Operator(String),
    String(String),
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    OpenParen,
    CloseParen,
    Colon,
    ColonEquals,
    QuestionMark,
    Let,
    Var,
    Do,
    Set,
    On,
    Return,
    Import,
    Export,
    SelfRef,
    If,
    Then,
    Else,
    End,
    True,
    False,
    EndOfInput,
}

impl Token {
    pub fn is_operator(ch: char) -> bool {
        OPERATORS.with(|set| set.contains(&ch))
    }

    pub fn from_ident(str: String) -> Token {
        KEYWORD_TOKENS
            .with(|pair| pair.clone())
            .0
            .get(&str)
            .cloned()
            .unwrap_or(Token::Identifier(str))
    }

    fn to_keyword(&self) -> Option<String> {
        KEYWORD_TOKENS
            .with(|pair| pair.clone())
            .1
            .get(self)
            .map(|s| s.to_string())
    }

    pub fn key_part(self) -> Option<String> {
        match self {
            Token::Identifier(key) => Some(key),
            Token::Operator(key) => Some(key),
            Token::Integer(num) => Some(num.to_string()),
            tok => tok.to_keyword(),
        }
    }

    pub fn with_source(self, source: Source) -> TokenWithSource {
        TokenWithSource {
            token: self,
            source,
        }
    }
}
type KeywordTokens = Rc<(HashMap<String, Token>, HashMap<Token, String>)>;

fn keyword_tokens() -> KeywordTokens {
    let pairs = vec![
        (Token::Let, "let"),
        (Token::Var, "var"),
        (Token::Set, "set"),
        (Token::Do, "do"),
        (Token::On, "on"),
        (Token::Return, "return"),
        (Token::SelfRef, "self"),
        (Token::Import, "import"),
        (Token::Export, "export"),
        (Token::If, "if"),
        (Token::Then, "then"),
        (Token::Else, "else"),
        (Token::End, "end"),
        (Token::True, "true"),
        (Token::False, "false"),
    ];

    Rc::new((
        pairs
            .iter()
            .map(|(token, str)| (str.to_string(), token.clone()))
            .collect(),
        pairs
            .iter()
            .map(|(token, str)| (token.clone(), str.to_string()))
            .collect(),
    ))
}

thread_local! {
  static KEYWORD_TOKENS: KeywordTokens = keyword_tokens();
  static OPERATORS: HashSet<char> = HashSet::from_iter("~!@$%^&*-+=|/.,<>".chars());
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Source {
    index: usize,
    length: usize,
}

impl Source {
    pub fn new(index: usize, length: usize) -> Self {
        Self { index, length }
    }
    pub fn in_context(&self, source: &str) -> SourceContext {
        let mut line_number = 1;
        let mut line_start = 0;
        for (i, char) in source.chars().enumerate().take(self.index) {
            if char == '\n' {
                line_number += 1;
                line_start = i;
            }
        }
        let max_context_padding = 10;
        let excerpt_start = line_start.max(self.index - max_context_padding);
        let mut excerpt_end = self.index + max_context_padding;
        for (i, char) in source.chars().enumerate().skip(self.index) {
            if char == '\n' {
                excerpt_end = i;
                break;
            }
        }

        let context = source[excerpt_start..excerpt_end].to_string();

        SourceContext {
            context,
            line: line_number,
            column: self.index - line_start,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceContext {
    context: String,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
pub struct TokenWithSource {
    pub token: Token,
    pub source: Source,
}
