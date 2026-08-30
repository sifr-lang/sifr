use crate::{ComponentError, ComponentErrorKind, DiagnosticRegistry};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentIdentity {
    pub package: String,
    pub processor: String,
    pub version: Version,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolRange {
    pub minimum: u16,
    pub maximum: u16,
}

impl ProtocolRange {
    #[must_use]
    pub const fn contains(self, version: u16) -> bool {
        self.minimum <= version && version <= self.maximum
    }

    pub fn validate(self) -> Result<(), ComponentError> {
        if self.minimum == 0 || self.minimum > self.maximum {
            return Err(ComponentError::new(
                ComponentErrorKind::Registration,
                "component protocol range is invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRegistration {
    pub identity: ComponentIdentity,
    pub protocol: ProtocolRange,
    pub artifact: String,
    pub diagnostics: DiagnosticRegistry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRequirement {
    pub identity: ComponentIdentity,
    pub protocol_major: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedComponent {
    pub registration: ComponentRegistration,
    pub bytes: Vec<u8>,
}

pub fn resolve_component(
    requirement: &ComponentRequirement,
    candidates: impl IntoIterator<Item = ResolvedComponent>,
) -> Result<ResolvedComponent, ComponentError> {
    let mut matches = candidates
        .into_iter()
        .filter(|candidate| candidate.registration.identity == requirement.identity)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(ComponentError::new(
            ComponentErrorKind::Registration,
            format!(
                "component resolution requires one exact candidate; found {}",
                matches.len()
            ),
        ));
    }
    let candidate = matches.pop().ok_or_else(|| {
        ComponentError::new(
            ComponentErrorKind::Registration,
            "component candidate is absent",
        )
    })?;
    candidate.registration.protocol.validate()?;
    DiagnosticRegistry::compiler()
        .validate_with(std::slice::from_ref(&candidate.registration.diagnostics))?;
    if !candidate
        .registration
        .protocol
        .contains(requirement.protocol_major)
    {
        return Err(ComponentError::new(
            ComponentErrorKind::ProtocolVersion,
            "component does not support the required protocol without downgrade",
        ));
    }
    verify_component_hash(&candidate.registration.identity.sha256, &candidate.bytes)?;
    Ok(candidate)
}

pub(crate) fn verify_component_hash(expected: &str, bytes: &[u8]) -> Result<(), ComponentError> {
    let actual = hex_digest(Sha256::digest(bytes).as_slice());
    if expected != actual {
        return Err(ComponentError::new(
            ComponentErrorKind::Integrity,
            format!("component hash mismatch: expected {expected}, found {actual}"),
        ));
    }
    Ok(())
}

#[must_use]
pub(crate) fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
