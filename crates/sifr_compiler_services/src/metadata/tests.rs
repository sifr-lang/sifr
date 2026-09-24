use super::*;
use sifr_identity::CompilerIdentity;
use std::path::Path;

#[test]
fn stale_or_corrupt_override_retries() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let scratch = tempfile::tempdir().unwrap();
    let identity = CompilerIdentity::for_test(vec![("metadata-test", "c02a0")], "override-retry");
    let target = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-pc-windows-msvc"
        } else {
            "x86_64-pc-windows-msvc"
        }
    } else if cfg!(target_arch = "aarch64") {
        "aarch64-unknown-linux-gnu"
    } else {
        "x86_64-unknown-linux-gnu"
    };
    let never_cancel = || false;
    let prepared = ensure_development_metadata(
        &identity,
        &source_root,
        target,
        scratch.path(),
        &never_cancel,
    )
    .unwrap();
    let override_path = scratch.path().join("override.sifrmeta");
    std::fs::write(&override_path, b"corrupt metadata").unwrap();
    assert!(
        validate_development_metadata(&identity, &source_root, target, &override_path).is_err()
    );
    std::fs::copy(&prepared.path, &override_path).unwrap();
    let accepted =
        validate_development_metadata(&identity, &source_root, target, &override_path).unwrap();
    assert_eq!(accepted.metadata_id, prepared.metadata_id);
    let stale_identity =
        CompilerIdentity::for_test(vec![("metadata-test", "c02a0")], "stale-owner");
    assert!(
        validate_development_metadata(&stale_identity, &source_root, target, &override_path)
            .is_err()
    );
    assert_eq!(
        validate_development_metadata(&identity, &source_root, target, &override_path)
            .unwrap()
            .metadata_id,
        prepared.metadata_id
    );
}
