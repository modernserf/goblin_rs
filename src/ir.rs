pub enum Value {
    Integer(u64),
}

pub enum IR {
    Constant(Value),
}
