//! Explicit caller authority for diagnostics in documents outside the template.
use crate::{ComponentError, ComponentErrorKind, ContextArtifact, SourceSpan};
use sha2::{Digest, Sha256};

pub const DIAGNOSTIC_SOURCE_LOCATIONS_KIND: &str = "sifr.compiler.diagnostic-source-locations";

pub fn diagnostic_source_artifact(
    identity: &str,
    locations: &[SourceSpan],
) -> Result<ContextArtifact, ComponentError> {
    let mut locations = locations.to_vec();
    locations.sort_by(|a, b| (&a.document, a.start, a.end).cmp(&(&b.document, b.start, b.end)));
    locations.dedup();
    let payload = serde_json::to_vec(&locations).map_err(|error| envelope(error.to_string()))?;
    Ok(ContextArtifact {
        kind: DIAGNOSTIC_SOURCE_LOCATIONS_KIND.into(),
        identity: identity.into(),
        format_version: 1,
        fingerprint: crate::registration::hex_digest(Sha256::digest(&payload).as_slice()),
        payload,
    })
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn diagnostic_source_locations(
    artifact: &ContextArtifact,
    maximum: usize,
) -> Result<Vec<SourceSpan>, ComponentError> {
    if artifact.format_version != 1
        || artifact.fingerprint
            != crate::registration::hex_digest(Sha256::digest(&artifact.payload).as_slice())
    {
        return Err(envelope(
            "diagnostic source artifact version or fingerprint is invalid",
        ));
    }
    let locations: Vec<SourceSpan> = serde_json::from_slice(&artifact.payload)
        .map_err(|error| envelope(format!("invalid diagnostic source locations: {error}")))?;
    if locations.len() > maximum {
        return Err(ComponentError::new(
            ComponentErrorKind::ResourceLimit,
            "diagnostic source location limit exceeded",
        ));
    }
    if locations
        .iter()
        .any(|span| span.document.is_empty() || span.start > span.end)
        || locations.windows(2).any(|spans| {
            (&spans[0].document, spans[0].start, spans[0].end)
                >= (&spans[1].document, spans[1].start, spans[1].end)
        })
    {
        return Err(envelope(
            "diagnostic source locations must be valid, unique and canonical",
        ));
    }
    Ok(locations)
}

fn envelope(message: impl Into<String>) -> ComponentError {
    ComponentError::new(ComponentErrorKind::ProtocolEnvelope, message)
}
