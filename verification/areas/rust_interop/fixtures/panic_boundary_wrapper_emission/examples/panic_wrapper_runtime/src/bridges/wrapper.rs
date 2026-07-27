use std::fmt;

use sifr_runtime::interop::RustPanicErrorBridge;

#[derive(Debug)]
pub struct PanicBridgeError {
    message: String,
}

impl PanicBridgeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PanicBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PanicBridgeError {}

pub fn may_panic(input: &str) -> Result<String, PanicBridgeError> {
    match input {
        "panic" => panic!("private panic payload must be redacted"),
        "error" => Err(PanicBridgeError::new("ordinary bridge error")),
        value => Ok(format!("ok:{value}")),
    }
}

pub fn map_panic(error: RustPanicErrorBridge) -> PanicBridgeError {
    PanicBridgeError::new(format!("mapped: {}", error.message()))
}

pub fn mapper_panics(_error: RustPanicErrorBridge) -> PanicBridgeError {
    panic!("private mapper panic payload must be redacted")
}

pub fn invalid_mapper(_error: RustPanicErrorBridge, _unexpected: &str) -> PanicBridgeError {
    PanicBridgeError::new("invalid mapper must never execute")
}
