use serde::Serialize;

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

pub(in crate::graph) fn digest_serializable<T: Serialize>(
    value: &T,
) -> Result<GraphDigest, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    Ok(GraphDigest {
        algorithm: "fnv1a64",
        hex: format!("{:016x}", fnv1a64(&bytes)),
    })
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::digest_serializable;
    use serde::Serialize;
    use serde::ser::{Error as _, Serializer};

    struct SerializationFailure;

    impl Serialize for SerializationFailure {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("intentional digest serialization failure"))
        }
    }

    #[test]
    fn serialization_failure_is_returned() {
        let error = digest_serializable(&SerializationFailure)
            .expect_err("serialization failure must not produce a digest");

        assert!(
            error
                .to_string()
                .contains("intentional digest serialization failure")
        );
    }
}
