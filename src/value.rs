#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Integer(u64),
}
