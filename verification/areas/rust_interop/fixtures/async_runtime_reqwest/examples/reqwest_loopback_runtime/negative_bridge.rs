use std::fmt;

#[derive(Debug)]
pub struct HttpBridgeError {
    message: String,
}

impl fmt::Display for HttpBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for HttpBridgeError {}

pub async fn request_roundtrip(
    payload: &str,
    _mode: &str,
) -> Result<String, HttpBridgeError> {
    let runtime = runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| HttpBridgeError {
            message: error.to_string(),
        })?;
    runtime.block_on(async { Ok(payload.to_string()) })
}

pub fn runtime_snapshot() -> String {
    "nested-runtime".to_string()
}

// Rust imports apply to their containing module independent of declaration
// order. The policy audit must fail closed when a module re-export exposes the
// Tokio runtime through a non-Tokio glob.
mod runtime_exports {
    pub use tokio::runtime;
}
use runtime_exports::*;
