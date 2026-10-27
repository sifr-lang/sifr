#![allow(clippy::expect_used)]
use super::*;

#[test]
fn all_bundled_sources_load_with_the_canonical_policy() {
    let engine = engine(policy::DEFAULT_STACK).expect("ordinary Engine");
    // Cross-target construction intentionally supplies no host-native images.
    if !generated::NATIVE_BUNDLE {
        assert!(generated::SOURCE_HASHES.is_empty());
        return;
    }
    assert_eq!(generated::SOURCE_HASHES.len(), 8);
    for hash in generated::SOURCE_HASHES {
        assert!(
            load_embedded(&engine, hash, policy::DEFAULT_STACK)
                .expect("trusted embedded image")
                .is_some()
        );
    }
}

#[test]
fn different_stack_policy_cannot_select_an_embedded_image() {
    let stack = policy::DEFAULT_STACK / 2;
    let engine = engine(stack).expect("valid smaller stack");
    for hash in generated::SOURCE_HASHES {
        assert!(
            load_embedded(&engine, hash, stack)
                .expect("policy miss")
                .is_none()
        );
    }
}

#[test]
fn unknown_source_hash_cannot_select_an_embedded_image() {
    let engine = engine(policy::DEFAULT_STACK).expect("ordinary Engine");
    assert!(
        load_embedded(&engine, &"0".repeat(64), policy::DEFAULT_STACK)
            .expect("source miss")
            .is_none()
    );
}

#[test]
fn engine_policy_is_deterministic_and_has_no_ambient_imports() {
    let engine = engine(policy::DEFAULT_STACK).expect("ordinary Engine");
    for hash in generated::SOURCE_HASHES {
        let component = load_embedded(&engine, hash, policy::DEFAULT_STACK)
            .expect("trusted image")
            .expect("registered source");
        assert_eq!(component.component_type().imports(&engine).count(), 0);
    }
}
