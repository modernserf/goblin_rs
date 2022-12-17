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
            .unwrap_or_else(|| Token::Identifier(str))
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
            tok => {
                if let Some(str) = tok.to_keyword() {
                    Some(str)
                } else {
                    None
                }
            }
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
