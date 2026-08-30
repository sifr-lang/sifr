use std::fmt;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeCodecIdentity(String);

impl RuntimeCodecIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, ParameterError> {
        let value = value.into();
        if !valid_identity(&value) {
            return Err(ParameterError::InvalidTypeIdentity);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq)]
pub enum OwnedSqlValue {
    Null,
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    ExactInteger(String),
    Text(String),
    Bytes(Arc<[u8]>),
    Sequence(Vec<OwnedSqlValue>),
    Encoded {
        type_identity: String,
        payload: Arc<[u8]>,
    },
}

impl fmt::Debug for OwnedSqlValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("Null"),
            Self::Bool(_) => formatter.write_str("Bool(<redacted>)"),
            Self::Signed(_) => formatter.write_str("Signed(<redacted>)"),
            Self::Unsigned(_) => formatter.write_str("Unsigned(<redacted>)"),
            Self::Float(_) => formatter.write_str("Float(<redacted>)"),
            Self::ExactInteger(value) => formatter
                .debug_struct("ExactInteger")
                .field("len", &value.len())
                .finish(),
            Self::Text(value) => formatter
                .debug_struct("Text")
                .field("len", &value.len())
                .finish(),
            Self::Bytes(value) => formatter
                .debug_struct("Bytes")
                .field("len", &value.len())
                .finish(),
            Self::Sequence(values) => formatter
                .debug_struct("Sequence")
                .field("len", &values.len())
                .finish(),
            Self::Encoded {
                type_identity,
                payload,
            } => formatter
                .debug_struct("Encoded")
                .field("type_identity", type_identity)
                .field("len", &payload.len())
                .finish(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OwnedParameter {
    pub slot: u32,
    pub codec: RuntimeCodecIdentity,
    pub value: OwnedSqlValue,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoundParameters {
    values: Vec<OwnedParameter>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterError {
    DuplicateSlot,
    InvalidExactInteger,
    InvalidTypeIdentity,
}

impl BoundParameters {
    pub fn new(mut values: Vec<OwnedParameter>) -> Result<Self, ParameterError> {
        values.sort_by_key(|value| value.slot);
        if values.windows(2).any(|pair| pair[0].slot == pair[1].slot) {
            return Err(ParameterError::DuplicateSlot);
        }
        for parameter in &values {
            validate_value(&parameter.value)?;
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn as_slice(&self) -> &[OwnedParameter] {
        &self.values
    }

    #[must_use]
    pub fn into_values(self) -> Vec<OwnedParameter> {
        self.values
    }
}

fn validate_value(value: &OwnedSqlValue) -> Result<(), ParameterError> {
    match value {
        OwnedSqlValue::ExactInteger(value) => {
            let digits = value.strip_prefix('-').unwrap_or(value);
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(ParameterError::InvalidExactInteger);
            }
        }
        OwnedSqlValue::Sequence(values) => {
            for value in values {
                validate_value(value)?;
            }
        }
        OwnedSqlValue::Encoded { type_identity, .. }
            if type_identity.is_empty()
                || type_identity.len() > 160
                || type_identity.chars().any(char::is_control) =>
        {
            return Err(ParameterError::InvalidTypeIdentity);
        }
        _ => {}
    }
    Ok(())
}

fn valid_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
}

impl fmt::Display for ParameterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::DuplicateSlot => "bound SQL parameters contain a duplicate slot",
            Self::InvalidExactInteger => "bound exact integer is not canonical decimal text",
            Self::InvalidTypeIdentity => "encoded SQL parameter type identity is invalid",
        })
    }
}

impl std::error::Error for ParameterError {}
