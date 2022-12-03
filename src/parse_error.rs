#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    ExpectedEndOfInput,
    Expected(String),
    ExpectedPairGotKey(String),
    DuplicateKey(String),
    DuplicateHandler(String),
    DuplicateElseHandler,
    WithSource(Box<ParseError>, Source),
}

use ParseError::*;

use crate::{parser::Parse, source::Source};
impl ParseError {
    pub fn expected_end_of_input<T>() -> Parse<T> {
        Err(ExpectedEndOfInput)
    }
    pub fn expected<T>(name: &str) -> Parse<T> {
        Err(Expected(name.to_string()))
    }
    pub fn expected_pair_got_key<T>(key: &str) -> Parse<T> {
        Err(ExpectedPairGotKey(key.to_string()))
    }
    pub fn duplicate_key<T>(key: &str) -> Parse<T> {
        Err(DuplicateKey(key.to_string()))
    }
    pub fn duplicate_handler<T>(selector: &str) -> Parse<T> {
        Err(DuplicateHandler(selector.to_string()))
    }
    pub fn duplicate_else_handler<T>() -> Parse<T> {
        Err(DuplicateElseHandler)
    }
    pub fn with_source<T>(self, source: Source) -> Parse<T> {
        Err(WithSource(Box::new(self), source))
    }
}
