#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    ExpectedEndOfInput,
    Expected(String),
    ExpectedPairGotKey(String),
    DuplicateKey(String),
    DuplicateHandler(String),
    DuplicateElseHandler,
    InvalidSetBinding,
    InvalidSetInPlace,
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
    pub fn invalid_set_binding<T>() -> Parse<T> {
        Err(InvalidSetBinding)
    }
    pub fn invalid_set_in_place<T>() -> Parse<T> {
        Err(InvalidSetInPlace)
    }
    pub fn with_source(self, source: Source) -> Self {
        WithSource(Box::new(self), source)
    }
}
