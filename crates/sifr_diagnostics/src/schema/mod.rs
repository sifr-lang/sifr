use crate::render::DiagnosticEnvelope;
use schemars::{Schema, schema_for};

#[must_use]
pub fn diagnostic_schema() -> Schema {
    schema_for!(DiagnosticEnvelope)
}

#[must_use]
pub fn diagnostic_schema_pretty_json() -> String {
    serde_json::to_string_pretty(&diagnostic_schema())
        .unwrap_or_else(|err| panic!("failed to serialize diagnostics JSON schema: {err}"))
}
