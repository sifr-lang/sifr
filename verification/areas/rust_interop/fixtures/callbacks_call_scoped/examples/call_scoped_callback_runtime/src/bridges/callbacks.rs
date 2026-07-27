use std::fmt;

use sifr_runtime::interop::{CallScopedCallbackBridge, IndexMap, SifrIntBridge};

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

pub fn visit_converted(
    callback: CallScopedCallbackBridge<
        '_,
        (
            SifrIntBridge,
            Vec<SifrIntBridge>,
            IndexMap<String, SifrIntBridge>,
            Option<SifrIntBridge>,
        ),
        Result<SifrIntBridge, String>,
    >,
) -> Result<SifrIntBridge, CallbackBridgeError> {
    let mut mapping = IndexMap::new();
    mapping.insert("value".to_string(), SifrIntBridge::from(3_i64));
    callback
        .call((
            SifrIntBridge::from(1_i64),
            vec![SifrIntBridge::from(2_i64), SifrIntBridge::from(3_i64)],
            mapping,
            Some(SifrIntBridge::from(4_i64)),
        ))
        .map_err(CallbackBridgeError::new)
}
