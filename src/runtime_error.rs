use crate::{interpreter::SendEffect, value::Value};

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeError {
    DoesNotUnderstand(String),
    PrimitiveTypeError { expected: String, received: Value },
    InvalidArg { expected: String, received: Value },
    AssertionError(String),
    UnknownModule(String),
    IndexOutOfRange,
}

impl RuntimeError {
    pub fn does_not_understand(selector: &str) -> SendEffect {
        SendEffect::Error(RuntimeError::DoesNotUnderstand(selector.to_string()))
    }

    pub fn primitive_type_error(expected: &str, received: &Value) -> SendEffect {
        SendEffect::Error(RuntimeError::PrimitiveTypeError {
            expected: expected.to_string(),
            received: received.clone(),
        })
    }

    pub fn invalid_arg(expected: &str, received: &Value) -> SendEffect {
        SendEffect::Error(RuntimeError::InvalidArg {
            expected: expected.to_string(),
            received: received.clone(),
        })
    }

    pub fn assertion_error(assertion: &str) -> SendEffect {
        SendEffect::Error(RuntimeError::AssertionError(assertion.to_string()))
    }

    pub fn unknown_module(name: &str) -> SendEffect {
        SendEffect::Error(RuntimeError::UnknownModule(name.to_string()))
    }

    pub fn index_out_of_range() -> SendEffect {
        SendEffect::Error(RuntimeError::IndexOutOfRange)
    }
}
