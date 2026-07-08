use super::*;

#[test]
pub(crate) fn test_infer_dependencies_maps_sifr_stdlib_fs_calls_to_private_module() {
    let (stdlib_modules, required_crates) = infer_dependencies(
        "fn read_text(path: &String) {\n    sifr_stdlib::fs::read_text(path);\n}\n",
        &BTreeSet::new(),
        &BTreeSet::new(),
    );

    assert!(stdlib_modules.contains("_sifr.fs"));
    assert!(required_crates.is_empty());
}
