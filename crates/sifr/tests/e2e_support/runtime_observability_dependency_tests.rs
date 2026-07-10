use super::*;

#[test]
pub(crate) fn test_infer_dependencies_recognizes_runtime_observability_bridge() {
    let (modules, crates) = infer_dependencies(
        "sifr_stdlib::runtime_observability::emit_diagnostic(level, target, name, message)",
        &BTreeSet::new(),
        &BTreeSet::new(),
    );
    assert!(modules.contains("_sifr.runtime"));
    assert!(!crates.contains("metrics"));
    assert!(!crates.contains("tracing"));
}
