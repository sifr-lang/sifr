use super::*;

#[test]
pub(crate) fn test_generate_cargo_toml_tomllib_uses_stdlib_toml_feature() {
    let stdlib_modules = normalize_dependency_set(vec!["sifr.tomllib".to_string()].into_iter());
    let required_crates = BTreeSet::new();

    let cargo_toml = generate_cargo_toml(&stdlib_modules, &required_crates, "sifr_output");
    assert!(cargo_toml.contains("sifr_stdlib = { path = "));
    assert!(cargo_toml.contains("default-features = false"));
    assert!(cargo_toml.contains("features = [\"toml\"]"));
    assert!(!cargo_toml.contains("toml = { version"));
}

#[test]
pub(crate) fn test_generate_cargo_toml_json_uses_stdlib_json_feature() {
    let stdlib_modules = normalize_dependency_set(vec!["sifr.json".to_string()].into_iter());
    let required_crates = BTreeSet::new();

    let cargo_toml = generate_cargo_toml(&stdlib_modules, &required_crates, "sifr_output");
    assert!(cargo_toml.contains("sifr_stdlib = { path = "));
    assert!(cargo_toml.contains("default-features = false"));
    assert!(cargo_toml.contains("features = [\"json\"]"));
    assert!(!cargo_toml.contains("serde_json = "));
}
