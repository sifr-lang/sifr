use crate::{
    CodecContract, CodecIdentity, DatabaseType, NullCodecBehavior, PanicContainment, SifrType,
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CodecFunctionIdentity(String);

impl CodecFunctionIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, CodecBindingError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 240
            || !value.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-')
            })
        {
            return Err(CodecBindingError::new(
                "codec function identity is not a canonical symbol path",
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckedCodecBinding {
    pub identity: CodecIdentity,
    pub database_type: DatabaseType,
    pub sifr_type: SifrType,
    pub encode: CodecFunctionIdentity,
    pub decode: CodecFunctionIdentity,
    pub encode_error: String,
    pub decode_error: String,
    pub null_behavior: NullCodecBehavior,
    pub panic_containment: PanicContainment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodecEncoderSignature {
    pub input: SifrType,
    pub output: DatabaseType,
    pub owned: bool,
    pub fallible: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodecDecoderSignature {
    pub input: DatabaseType,
    pub output: SifrType,
    pub owned: bool,
    pub fallible: bool,
}

impl CheckedCodecBinding {
    pub fn checked(
        contract: &CodecContract,
        encode: CodecFunctionIdentity,
        decode: CodecFunctionIdentity,
        encoder: &CodecEncoderSignature,
        decoder: &CodecDecoderSignature,
    ) -> Result<Self, CodecBindingError> {
        if !encoder.owned
            || !encoder.fallible
            || !decoder.owned
            || !decoder.fallible
            || encoder.input != contract.sifr_type
            || encoder.output != contract.database_type
            || decoder.input != contract.database_type
            || decoder.output != contract.sifr_type
            || contract.encode_error.trim().is_empty()
            || contract.decode_error.trim().is_empty()
        {
            return Err(CodecBindingError::new(
                "custom codec functions must be owned, fallible, and exact inverses of the declared database identity",
            ));
        }
        if let DatabaseType::Custom { identity, codec } = &contract.database_type {
            if codec != &contract.identity || identity.as_str().trim().is_empty() {
                return Err(CodecBindingError::new(
                    "custom codec database identity does not match its codec contract",
                ));
            }
        }
        Ok(Self {
            identity: contract.identity.clone(),
            database_type: contract.database_type.clone(),
            sifr_type: contract.sifr_type.clone(),
            encode,
            decode,
            encode_error: contract.encode_error.clone(),
            decode_error: contract.decode_error.clone(),
            null_behavior: contract.null_behavior,
            panic_containment: contract.panic_containment,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodecBindingError {
    pub message: String,
}

impl CodecBindingError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CodecBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CodecBindingError {}
