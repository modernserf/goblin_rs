use crate::source::Source;

#[derive(Debug, PartialEq, Clone)]
pub enum Binding {
    Identifier(String, Source),
}
