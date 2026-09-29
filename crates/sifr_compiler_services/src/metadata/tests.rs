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

fn h02a0_artifact() -> (ProjectTypedArtifact, sifr_ir::MethodAuthority) {
    use sifr_ir::{
        BindingId, CallableIdentity, HirExpr, HirModule, MethodCallSource, MutableArgumentTarget,
        MutableReceiverTarget, Place,
    };
    use sifr_type_system::{ReceiverConvention, Type};
    let source = sifr_frontend::persistence::CapturedSource {
        path: "/workspace/sample.sifr".into(),
        text: "value: int = 1\n".into(),
    };
    let authority = sifr_ir::MethodAuthority::Imported {
        declaration: CallableIdentity {
            module: "dependency".into(),
            owner: Some("Container".into()),
            symbol: "update".into(),
            generic_arguments: vec!["int".into()],
            signature: "(mut int) -> int".into(),
        },
    };
    let range = ruff_text_size::TextRange::new(2.into(), 10.into());
    let place = Place {
        root: BindingId(4),
        projections: vec![],
    };
    let call = HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "receiver".into(),
            binding_id: Some(BindingId(4)),
            ty: Type::Int,
        }),
        method: "update".into(),
        args: vec![HirExpr::IntLiteral(1)],
        authority: authority.clone(),
        receiver_convention: Some(ReceiverConvention::MutableBorrow),
        receiver_target: Some(MutableReceiverTarget::Place(place.clone())),
        mutable_arg_places: vec![Some(MutableArgumentTarget::Place(place))],
        source: Some(MethodCallSource {
            call_range: range,
            receiver_range: range,
            arg_ranges: vec![range],
        }),
        ty: Type::Int,
    };
    let hir = HirModule {
        functions: vec![],
        classes: vec![],
        imports: vec![],
        constants: vec![("value".into(), Type::Int, call)],
        generic_functions: Default::default(),
        type_param_bounds: Default::default(),
    };
    let compatibility = wire::Compatibility {
        compiler: [1; 32],
        semantic_target: [2; 32],
        stdlib_inputs: [3; 32],
    };
    let artifact = encode_project_results(
        "package@1/source-A",
        &std::collections::BTreeMap::from([(
            "sample".into(),
            ProjectModuleInput {
                source: &source,
                input_identity: source.identity(),
                hir: &hir,
            },
        )]),
        &sifr_lowering::ExternalDefs::default(),
        compatibility,
    )
    .unwrap();
    (artifact, authority)
}

#[test]
fn h02a0_hir_authority_roundtrip() {
    use sifr_ir::{HirExpr, MutableArgumentTarget, MutableReceiverTarget, Place};
    let (artifact, expected_authority) = h02a0_artifact();
    let compatibility = wire::Compatibility {
        compiler: [1; 32],
        semantic_target: [2; 32],
        stdlib_inputs: [3; 32],
    };
    let decoded = reader::decode_project_results(&artifact, compatibility).unwrap();
    let (
        _,
        _,
        HirExpr::MethodCall {
            authority,
            receiver_convention,
            receiver_target,
            mutable_arg_places,
            source: Some(source),
            ..
        },
    ) = &decoded["sample"].0.constants[0]
    else {
        panic!("method call lost");
    };
    assert_eq!(*authority, expected_authority);
    assert_eq!(
        *receiver_convention,
        Some(sifr_type_system::ReceiverConvention::MutableBorrow)
    );
    assert!(matches!(
        receiver_target,
        Some(MutableReceiverTarget::Place(Place { .. }))
    ));
    assert!(matches!(
        mutable_arg_places.as_slice(),
        [Some(MutableArgumentTarget::Place(_))]
    ));
    assert_eq!(
        source.call_range,
        ruff_text_size::TextRange::new(2.into(), 10.into())
    );
    assert_eq!(source.receiver_range, source.call_range);
    assert_eq!(source.arg_ranges, vec![source.call_range]);
    let reencoded = encode_project_results(
        "package@1/source-A",
        &std::collections::BTreeMap::from([(
            "sample".into(),
            ProjectModuleInput {
                source: artifact.sources.values().next().unwrap(),
                input_identity: artifact.sources.values().next().unwrap().identity(),
                hir: &decoded["sample"].0,
            },
        )]),
        &decoded["sample"].1,
        compatibility,
    )
    .unwrap();
    assert_eq!(artifact.bytes, reencoded.bytes);
}

#[test]
fn h02a0_corrupt_authority_rejects() {
    let (artifact, _) = h02a0_artifact();
    let compatibility = wire::Compatibility {
        compiler: [1; 32],
        semantic_target: [2; 32],
        stdlib_inputs: [3; 32],
    };
    for malformed in [
        r#""FutureAuthority""#,
        r#"{"Imported":{}}"#,
        r#"{"RustAdapted":{"declaration":null}}"#,
    ] {
        assert!(serde_json::from_str::<wire::MethodAuthority>(malformed).is_err());
    }
    let mut corrupt = artifact;
    let final_byte = corrupt.bytes.last_mut().unwrap();
    *final_byte ^= 0xff;
    assert!(reader::decode_project_results(&corrupt, compatibility).is_err());
}
