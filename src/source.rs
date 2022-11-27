#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Source {
    pub start: usize,
    pub length: usize,
}

impl Source {
    pub fn new(start: usize, length: usize) -> Self {
        Source { start, length }
    }
}
