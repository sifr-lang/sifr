use crate::{SchemaContractError, SchemaContractErrorKind, SchemaIr};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaFingerprint(String);

impl SchemaFingerprint {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize)]
struct CanonicalSchema<'a> {
    format_version: u32,
    provider: &'a crate::ProviderIdentity,
    dialect: &'a crate::DialectIdentity,
    objects: Vec<CanonicalObject<'a>>,
}

#[derive(Serialize)]
struct CanonicalObject<'a> {
    identity: &'a crate::ObjectId,
    kind: crate::SchemaObjectKind,
    semantic: &'a std::collections::BTreeMap<String, crate::SemanticValue>,
    dependencies: &'a std::collections::BTreeSet<crate::ObjectId>,
}

pub fn schema_fingerprint(schema: &SchemaIr) -> Result<SchemaFingerprint, SchemaContractError> {
    let canonical = CanonicalSchema {
        format_version: schema.format_version,
        provider: &schema.provider,
        dialect: &schema.dialect,
        objects: schema
            .objects
            .values()
            .map(|object| CanonicalObject {
                identity: &object.identity,
                kind: object.kind,
                semantic: &object.semantic,
                dependencies: &object.dependencies,
            })
            .collect(),
    };
    let bytes = serde_json::to_vec(&canonical).map_err(|error| {
        SchemaContractError::new(
            SchemaContractErrorKind::Serialization,
            format!("cannot serialize canonical schema: {error}"),
        )
    })?;
    Ok(SchemaFingerprint(lower_hex(&Sha256::digest(bytes))))
}

pub(crate) fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}
