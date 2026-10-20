use super::{cargo_edges, cargo_inventory, registry_audit};
use crate::support::TestUnwrap as _;
use cargo_inventory::pinned_graphs::{self, POLICY};
use std::collections::BTreeMap;

fn fixture() -> (serde_json::Value, BTreeMap<String, Vec<u8>>) {
    (
        serde_json::from_str(POLICY).test_unwrap("policy"),
        pinned_graphs::inputs(),
    )
}

#[test]
fn pinned_graphs_authenticate_every_registered_input_and_registry_identity() {
    let (policy, inputs) = fixture();
    let approved = pinned_graphs::validate(&policy, &inputs).test_unwrap("approved graphs");
    assert_eq!(approved.len(), 24);
    assert_eq!(
        policy["registry"].as_array().test_unwrap("registry").len(),
        371
    );
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "crates/sifr_codegen/Cargo.toml",
        "verification/new_tool/Cargo.lock",
    ] {
        assert!(!approved.contains(path), "{path}");
    }
}

#[test]
fn pinned_graphs_reject_each_missing_or_changed_manifest_and_lock() {
    let (policy, inputs) = fixture();
    for path in inputs.keys() {
        let mut missing = inputs.clone();
        missing.remove(path);
        assert!(
            pinned_graphs::validate(&policy, &missing).is_err(),
            "{path}"
        );
        let mut changed = inputs.clone();
        changed
            .get_mut(path)
            .test_unwrap("input")
            .extend_from_slice(b"\n# unreviewed edit\n");
        assert!(
            pinned_graphs::validate(&policy, &changed).is_err(),
            "{path}"
        );
    }
}

#[test]
fn pinned_graphs_reject_extra_inputs_and_product_scope_expansion() {
    let (policy, inputs) = fixture();
    for path in [
        "verification/fuzz/new_owner/Cargo.toml",
        "verification/unregistered/Cargo.lock",
        "Cargo.lock",
        "verification/fuzz/../../Cargo.lock",
    ] {
        let mut changed = inputs.clone();
        changed.insert(path.into(), b"[workspace]\n".to_vec());
        assert!(
            pinned_graphs::validate(&policy, &changed).is_err(),
            "{path}"
        );
        if !path.starts_with("verification/fuzz/new_owner/") {
            let mut expanded = policy.clone();
            expanded["files"][path] = sifr_sysroot::sha256_hex(&changed[path]).into();
            assert!(
                pinned_graphs::validate(&expanded, &changed).is_err(),
                "{path}"
            );
        }
    }
    let mut roots = policy.clone();
    roots["roots"]
        .as_array_mut()
        .test_unwrap("roots")
        .push("crates".into());
    assert!(pinned_graphs::validate(&roots, &inputs).is_err());
    let mut removed = policy.clone();
    removed["files"]
        .as_object_mut()
        .test_unwrap("files")
        .remove("verification/fuzz/Cargo.lock");
    let mut removed_inputs = inputs.clone();
    removed_inputs.remove("verification/fuzz/Cargo.lock");
    assert!(pinned_graphs::validate(&removed, &removed_inputs).is_err());
}

#[test]
fn pinned_registry_rejects_missing_extra_duplicate_and_corrupt_authority() {
    let (policy, inputs) = fixture();
    let mut missing = policy.clone();
    missing["registry"]
        .as_array_mut()
        .test_unwrap("rows")
        .remove(0);
    assert!(pinned_graphs::validate(&missing, &inputs).is_err());
    let mut duplicate = policy.clone();
    let row = duplicate["registry"][0].clone();
    duplicate["registry"]
        .as_array_mut()
        .test_unwrap("rows")
        .push(row);
    assert!(pinned_graphs::validate(&duplicate, &inputs).is_err());
    for (field, value) in [
        ("name", "unadmitted-package"),
        ("version", "99.0.0"),
        (
            "checksum",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
        ("source_url", "https://example.invalid/index"),
        ("response_sha256", "bad"),
        ("observed_at", ""),
    ] {
        let mut changed = policy.clone();
        changed["registry"][0][field] = value.into();
        assert!(
            pinned_graphs::validate(&changed, &inputs).is_err(),
            "{field}"
        );
    }
}

#[test]
fn changed_lock_cannot_pass_with_only_a_new_input_hash() {
    let (policy, inputs) = fixture();
    let path = "verification/fuzz/Cargo.lock";
    for (field, value) in [
        ("version", "99.0.0"),
        ("source", "registry+https://example.invalid/index"),
        (
            "checksum",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ),
    ] {
        let mut lock: toml::Value =
            toml::from_str(std::str::from_utf8(&inputs[path]).test_unwrap("utf8"))
                .test_unwrap("lock");
        let package = lock["package"]
            .as_array_mut()
            .test_unwrap("packages")
            .iter_mut()
            .find(|package| {
                package["name"].as_str() == Some("syn")
                    && package["version"].as_str() == Some("3.0.6")
            })
            .test_unwrap("pinned syn");
        package[field] = value.into();
        let mut changed_inputs = inputs.clone();
        let bytes = toml::to_string(&lock)
            .test_unwrap("lock source")
            .into_bytes();
        let mut changed_policy = policy.clone();
        changed_policy["files"][path] = sifr_sysroot::sha256_hex(&bytes).into();
        changed_inputs.insert(path.into(), bytes);
        assert!(
            pinned_graphs::validate(&changed_policy, &changed_inputs).is_err(),
            "{field}"
        );
    }
}

#[test]
fn pinned_graph_approval_does_not_relax_product_version_policy() {
    assert!(!registry_audit::requirement_matches_latest("1", "1.0.150"));
    assert!(!registry_audit::requirement_matches_latest(
        "=1.0.102", "1.0.104"
    ));
    let lock: toml::Value = toml::from_str("[[package]]\nname = 'syn'\nversion = '3.0.6'\nsource = 'registry+https://github.com/rust-lang/crates.io-index'\n").test_unwrap("product mutation");
    let packages = cargo_edges::packages(&lock).test_unwrap("packages");
    assert!(cargo_edges::current_edge(packages, "syn", "syn", "3.0.5").is_err());
}
