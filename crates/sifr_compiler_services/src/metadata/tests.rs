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

#[test]
fn incompatible_identity_rejects_provider() {
    let scratch = tempfile::tempdir().unwrap();
    let first =
        crate::CompilerContext::for_test_tokens(vec![("metadata-reader", "c02a1")], "identity-a")
            .with_cache_root(scratch.path().join("cache-a"));
    let selected = first.metadata_provider().unwrap();
    let override_path = scratch.path().join("selected.sifrmeta");
    std::fs::copy(&selected.metadata.path, &override_path).unwrap();
    let incompatible =
        crate::CompilerContext::for_test_tokens(vec![("metadata-reader", "c02a1")], "identity-b")
            .with_cache_root(scratch.path().join("cache-b"))
            .with_metadata_override(override_path);
    assert!(incompatible.metadata_provider().is_err());
    assert!(!first.shares_metadata_generation(&incompatible));
    assert_eq!(
        first.metadata_provider().unwrap().metadata.metadata_id,
        selected.metadata.metadata_id,
    );
}

#[test]
fn replaced_cache_owner_does_not_reuse_generation() {
    let scratch = tempfile::tempdir().unwrap();
    let original =
        crate::CompilerContext::for_test_tokens(vec![("metadata-reader", "c02a1")], "cache-owner")
            .with_cache_root(scratch.path().join("first"));
    let first = original.metadata_provider().unwrap();
    let replacement = original
        .clone()
        .with_cache_root(scratch.path().join("second"));
    assert!(!original.shares_metadata_generation(&replacement));
    let second = replacement.metadata_provider().unwrap();
    assert!(!std::sync::Arc::ptr_eq(&first, &second));
    assert_ne!(first.metadata.path, second.metadata.path);
    assert_eq!(first.metadata.metadata_id, second.metadata.metadata_id);
    assert!(std::sync::Arc::ptr_eq(
        &first,
        &original.metadata_provider().unwrap(),
    ));
}
