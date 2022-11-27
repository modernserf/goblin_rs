#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Source {
    pub start: usize,
    pub length: usize,
}

impl Source {
    pub fn new(start: usize, length: usize) -> Self {
        Source { start, length }
    }
    // Show the relevant span of source code
    //          ^^^^^^^^^^^^^ like this
    pub fn in_context(code: &str) {
        unimplemented!()
    }
    // get the start & end sources & make one big source
    pub fn span(&self, other: Source) -> Source {
        if other.start < self.start {
            panic!("sources out of order")
        }
        Source {
            start: self.start,
            length: other.length + (other.start - self.start),
        }
    }
}
