use crate::protocol::EmbeddedPlan;
use crate::registration::hex_digest;
use crate::{ComponentError, ComponentErrorKind};
use sha2::{Digest, Sha256};

pub fn compute_plan_fingerprint(plan: &EmbeddedPlan) -> Result<String, ComponentError> {
    let mut canonical = plan.clone();
    canonical.stable_fingerprint.clear();
    let bytes = serde_json::to_vec(&canonical).map_err(|error| {
        ComponentError::new(ComponentErrorKind::ProtocolEnvelope, error.to_string())
    })?;
    Ok(hex_digest(Sha256::digest(bytes).as_slice()))
}
