use serde::Serialize;

pub use super::digest_build_cache::{
    digest_package_build_cache_inputs, digest_python_environment_probe, PackageBuildCacheInputs,
};
pub use super::digest_cargo_metadata::digest_graph_inputs;
pub use super::digest_package_graph::digest_package_graph;
pub use super::digest_source_map::{digest_package_source_map, digest_package_source_snapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphDigest {
    pub algorithm: &'static str,
    pub hex: String,
}

pub(in crate::graph) fn digest_serializable<T: Serialize>(value: &T) -> GraphDigest {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    GraphDigest {
        algorithm: "fnv1a64",
        hex: format!("{:016x}", fnv1a64(&bytes)),
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
