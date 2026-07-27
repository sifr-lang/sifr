use std::fmt;

use sifr_runtime::interop::CallScopedCallbackBridge;

#[derive(Debug)]
pub struct CallbackBridgeError {
    message: String,
}

impl CallbackBridgeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CallbackBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CallbackBridgeError {}

pub fn visit(
    mode: &str,
    callback: CallScopedCallbackBridge<'_, (String,), Result<(), String>>,
) -> Result<String, CallbackBridgeError> {
    match mode {
        "safe" => {
            callback
                .call(("first".to_string(),))
                .map_err(CallbackBridgeError::new)?;
            callback
                .call(("second".to_string(),))
                .map_err(CallbackBridgeError::new)?;
            Ok("first,second".to_string())
        }
        "error" => {
            callback
                .call(("reject".to_string(),))
                .map_err(CallbackBridgeError::new)?;
            Ok("unexpected callback success".to_string())
        }
        "panic" => {
            callback
                .call(("panic".to_string(),))
                .map_err(CallbackBridgeError::new)?;
            Ok("unexpected callback success".to_string())
        }
        other => Err(CallbackBridgeError::new(format!(
            "unsupported callback mode: {other}"
        ))),
    }
}
