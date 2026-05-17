use lsp_server::ErrorCode;
use serde_json::Value;
use std::error::Error;
use std::fmt;

pub(crate) type LspResult<T> = Result<T, LspError>;
pub(crate) type ServerResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub(crate) struct LspError {
    code: ErrorCode,
    message: String,
}

impl LspError {
    pub(crate) fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InvalidParams,
            message: message.into(),
        }
    }

    pub(crate) fn method_not_found(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::MethodNotFound,
            message: message.into(),
        }
    }

    pub(crate) fn request_cancelled(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::RequestCanceled,
            message: message.into(),
        }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::InternalError,
            message: message.into(),
        }
    }

    pub(crate) fn code(&self) -> i32 {
        self.code as i32
    }

    pub(crate) fn message(&self) -> String {
        self.message.clone()
    }
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for LspError {}

impl From<serde_json::Error> for LspError {
    fn from(error: serde_json::Error) -> Self {
        Self::invalid_params(format!("invalid LSP params: {error}"))
    }
}

pub(crate) fn required_string(value: &Value, path: &str) -> LspResult<String> {
    value
        .pointer(path)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| LspError::invalid_params(format!("missing string parameter at {path}")))
}

pub(crate) fn optional_i32(value: &Value, path: &str) -> LspResult<Option<i32>> {
    value
        .pointer(path)
        .map(|raw| {
            let number = raw.as_i64().ok_or_else(|| {
                LspError::invalid_params(format!("expected integer parameter at {path}"))
            })?;
            i32::try_from(number).map_err(|_| {
                LspError::invalid_params(format!("integer parameter at {path} is out of range"))
            })
        })
        .transpose()
}
