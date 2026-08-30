use crate::{SqlError, SqlErrorKind};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaEvidenceMode {
    Introspection,
    MigrationHead,
    SignedManifest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaStrictness {
    Exact,
    Compatible,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaProperty {
    pub identity: String,
    pub value: Option<String>,
}

impl SchemaProperty {
    pub fn new(identity: impl Into<String>, value: Option<String>) -> Result<Self, SqlError> {
        let property = Self {
            identity: identity.into(),
            value,
        };
        if property.identity.is_empty()
            || property.identity.len() > 512
            || property.identity.chars().any(char::is_control)
            || property
                .value
                .as_ref()
                .is_some_and(|value| value.len() > 16 * 1024 || value.chars().any(char::is_control))
        {
            return Err(SqlError::new(SqlErrorKind::SchemaContract));
        }
        Ok(property)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaDependencySlice {
    fingerprint: String,
    properties: BTreeMap<String, Option<String>>,
}

impl SchemaDependencySlice {
    pub fn new(
        fingerprint: impl Into<String>,
        properties: impl IntoIterator<Item = SchemaProperty>,
    ) -> Result<Self, SqlError> {
        let fingerprint = fingerprint.into();
        if !valid_fingerprint(&fingerprint) {
            return Err(SqlError::new(SqlErrorKind::SchemaContract));
        }
        let mut indexed = BTreeMap::new();
        for property in properties {
            if indexed.insert(property.identity, property.value).is_some() {
                return Err(SqlError::new(SqlErrorKind::SchemaContract));
            }
        }
        Ok(Self {
            fingerprint,
            properties: indexed,
        })
    }

    #[must_use]
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    #[must_use]
    pub fn properties(&self) -> &BTreeMap<String, Option<String>> {
        &self.properties
    }
}

pub fn verify_schema(
    strictness: SchemaStrictness,
    expected: &SchemaDependencySlice,
    observed: &SchemaDependencySlice,
) -> Result<(), SqlError> {
    match strictness {
        SchemaStrictness::Exact if expected.fingerprint != observed.fingerprint => {
            Err(SqlError::new(SqlErrorKind::SchemaContract))
        }
        SchemaStrictness::Exact => Ok(()),
        SchemaStrictness::Compatible => {
            for (identity, expected_value) in &expected.properties {
                if observed.properties.get(identity) != Some(expected_value) {
                    return Err(SqlError::new(SqlErrorKind::SchemaContract));
                }
            }
            Ok(())
        }
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
