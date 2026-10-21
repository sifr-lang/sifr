use super::*;
fn source_context() -> SemanticContext {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    SemanticContext::with_sysroot(
        CompilerIdentity::for_test(crate::compiled_input_tokens(), "semantic-context-tests"),
        sifr_sysroot::resolve_sysroot(Some(root)).unwrap(),
    )
}
#[test]
fn source_owner_is_complete_and_session_local() {
    let first = source_context();
    let second = source_context();
    assert!(!Arc::ptr_eq(&first.owner, &second.owner));
    assert!(Arc::ptr_eq(&first.owner, &first.clone().owner));
    let defs = first.external_defs().unwrap();
    assert!(
        defs.provider.is_none(),
        "source session must own checked definitions"
    );
    let inputs = metadata::inputs::Inputs::capture(
        first.identity(),
        &first.sysroot().unwrap().root,
        metadata::installed::target(),
    )
    .unwrap();
    let owner = first.owner.lock().unwrap().as_ref().unwrap().clone();
    let StdlibOwner::Source(checked) = owner.as_ref() else {
        panic!("source did not produce checked inventory")
    };
    let expected = inputs
        .sources
        .iter()
        .map(|source| source.module.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        checked
            .hir_modules
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        expected
    );
    first.external_defs().unwrap();
    assert!(Arc::ptr_eq(
        &owner,
        first.owner.lock().unwrap().as_ref().unwrap()
    ));
    assert!(second.owner.lock().unwrap().is_none());
}
#[test]
fn explicit_missing_or_corrupt_artifact_never_falls_back_to_sources() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("metadata");
    let context = source_context().with_metadata_override(path.clone());
    assert!(context.external_defs().is_err());
    assert!(context.owner.lock().unwrap().is_none());
    std::fs::write(&path, b"corrupt").unwrap();
    assert!(context.external_defs().is_err());
    assert!(context.owner.lock().unwrap().is_none());
}
#[test]
fn installed_descriptor_failure_does_not_check_source_or_memoize_failure() {
    let temp = tempfile::tempdir().unwrap();
    let original = source_context();
    let mut root = original.sysroot().unwrap().clone();
    root.root = temp.path().to_owned();
    root.manifest.target_triple = metadata::installed::target().to_string();
    let context = SemanticContext::with_sysroot(original.identity().clone(), root);
    let error = context.external_defs().unwrap_err();
    assert!(error[0].message.contains("required metadata descriptor"));
    assert!(context.owner.lock().unwrap().is_none());
    std::fs::create_dir_all(temp.path().join("lib/sifr")).unwrap();
    std::fs::write(temp.path().join("lib/sifr/stdlib.metadata.json"), b"{}").unwrap();
    let error = context.external_defs().unwrap_err();
    assert!(
        error[0]
            .message
            .contains("invalid installed metadata descriptor")
    );
    assert!(context.owner.lock().unwrap().is_none());
}
#[test]
fn override_releases_only_the_new_contexts_owner() {
    let original = source_context();
    original.external_defs().unwrap();
    let changed = original
        .clone()
        .with_metadata_override(PathBuf::from("/unavailable-metadata"));
    assert!(!Arc::ptr_eq(&original.owner, &changed.owner));
    assert!(original.owner.lock().unwrap().is_some());
    assert!(changed.owner.lock().unwrap().is_none());
}
