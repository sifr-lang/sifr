use serde::Serialize;
use sha2::{Digest as _, Sha256};

pub use super::digest_build_cache::{
    PackageBuildCacheInputs, digest_package_build_cache_inputs,
    digest_python_authoring_environment_probe, digest_python_environment_probe,
};
pub use super::digest_cargo_metadata::digest_graph_inputs;
pub use super::digest_package_graph::digest_package_graph;
pub use super::digest_source_map::{digest_package_source_map, digest_package_source_snapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphDigest {
    pub algorithm: &'static str,
    pub hex: String,
}

/// Persisted package identities use a separate schema and domain for each
/// authority. A length frame prevents domain/payload boundary ambiguity.
pub(in crate::graph) fn digest_serializable<T: Serialize>(
    domain: &'static str,
    value: &T,
) -> GraphDigest {
    // Serialization failure handling belongs to N02/F08. The concrete package
    // projections here contain only infallibly serializable values.
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    digest_bytes(domain, &bytes)
}

pub(in crate::graph) fn digest_bytes(domain: &'static str, payload: &[u8]) -> GraphDigest {
    let mut digest = Sha256::new();
    digest.update(b"sifr-package-identity\0");
    digest.update(1_u32.to_be_bytes());
    digest.update((domain.len() as u64).to_be_bytes());
    digest.update(domain.as_bytes());
    digest.update((payload.len() as u64).to_be_bytes());
    digest.update(payload);
    let bytes = digest.finalize();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    GraphDigest {
        algorithm: "sha256-framed-v1",
        hex,
    }
}

#[cfg(test)]
mod tests {
    use super::digest_bytes;

    #[test]
    fn framing_separates_domains_and_empty_payloads() {
        assert_ne!(digest_bytes("a", b"bc"), digest_bytes("ab", b"c"));
        assert_ne!(digest_bytes("a", b""), digest_bytes("b", b""));
        assert_ne!(digest_bytes("a", b""), digest_bytes("a", b"\0"));
    }
}
