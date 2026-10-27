//! Temporary V1co diagnostic. Not a distributable bundle implementation.
use std::hash::{Hash, Hasher};
use wasmtime::{Engine, component::Component};

pub(super) fn load(
    engine: &Engine,
    hash: &str,
    stack: usize,
) -> wasmtime::Result<Option<Component>> {
    if stack != 2097152 || compatibility_key(engine) != 3132910427983818482 {
        return Ok(None);
    }
    let bytes: &'static [u8] = match hash {
        "31e5ef13967fbc81f1e1ab71944d086d5464bbe36d5b7733b1660b5f9c63e0bc" => include_bytes!(
            "/data/sifr-local-gate-implementation-20261020/v1cn-embedded-probe/compiled/mysql.cwasm"
        ),
        "15b9341d788144a20fa4d2f798ddec1a17113a152909e3776855d06a5d1ddc7f" => include_bytes!(
            "/data/sifr-local-gate-implementation-20261020/v1cn-embedded-probe/compiled/postgresql.cwasm"
        ),
        "6307096cb3becf35ec2ed140e05d9b664136235dcee6a72411b2233032060ebb" => include_bytes!(
            "/data/sifr-local-gate-implementation-20261020/v1cn-embedded-probe/compiled/sqlite.cwasm"
        ),
        _ => return Ok(None),
    };
    // SAFETY: these immutable constants were produced by the recorded trusted
    // Engine::precompile_component invocations and verified before embedding.
    // ComponentHost authenticates current Wasm before selecting its exact hash.
    // No runtime path or caller-supplied native bytes can reach this operation.
    // Matching policy/Engine identity is required; deserialize errors propagate.
    unsafe { Component::deserialize(engine, bytes) }.map(Some)
}

fn compatibility_key(engine: &Engine) -> u64 {
    let mut state = std::collections::hash_map::DefaultHasher::new();
    engine.precompile_compatibility_hash().hash(&mut state);
    state.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentCompilation, ComponentHostLimits};
    const HASHES: &[&str] = &[
        "31e5ef13967fbc81f1e1ab71944d086d5464bbe36d5b7733b1660b5f9c63e0bc",
        "15b9341d788144a20fa4d2f798ddec1a17113a152909e3776855d06a5d1ddc7f",
        "6307096cb3becf35ec2ed140e05d9b664136235dcee6a72411b2233032060ebb",
    ];

    #[test]
    fn every_bundled_family_loads_with_ordinary_engine_policy() {
        let limits = ComponentHostLimits::default();
        let owner = ComponentCompilation::new(&limits).unwrap();
        for hash in HASHES {
            assert!(
                load(owner.engine(), hash, limits.max_wasm_stack_bytes)
                    .unwrap()
                    .is_some()
            );
        }
    }

    #[test]
    fn changed_policy_never_loads_an_embedded_image() {
        let limits = ComponentHostLimits {
            max_wasm_stack_bytes: 1024 * 1024,
            ..ComponentHostLimits::default()
        };
        let owner = ComponentCompilation::new(&limits).unwrap();
        for hash in HASHES {
            assert!(
                load(owner.engine(), hash, limits.max_wasm_stack_bytes)
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn unknown_source_hash_never_loads_an_embedded_image() {
        let limits = ComponentHostLimits::default();
        let owner = ComponentCompilation::new(&limits).unwrap();
        assert!(
            load(owner.engine(), &"0".repeat(64), limits.max_wasm_stack_bytes)
                .unwrap()
                .is_none()
        );
    }
}
