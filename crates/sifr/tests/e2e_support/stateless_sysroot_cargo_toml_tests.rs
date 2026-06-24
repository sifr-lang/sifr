use super::*;

#[test]
pub(crate) fn test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features() {
    let combined_modules = normalize_dependency_set(
        [
            "sifr.platform",
            "sifr.html",
            "_sifr.calendar",
            "sifr.uuid",
            "sifr.math",
            "sifr.hashlib",
        ]
        .map(str::to_string),
    );
    let required_crates = BTreeSet::new();
    let uuid_modules = normalize_dependency_set(vec!["_sifr.uuid".to_string()].into_iter());
    let uuid_toml = generate_cargo_toml(&uuid_modules, &required_crates, "sifr_output");
    assert!(uuid_toml.contains("sifr_stdlib = { path = "));
    assert!(uuid_toml.contains("default-features = false"));
    assert!(uuid_toml.contains("features = [\"uuid\"]"));
    assert!(!uuid_toml.contains("rand = "));
    assert!(!uuid_toml.contains("uuid = { version"));
    let hash_modules = normalize_dependency_set(vec!["sifr.hashlib".to_string()].into_iter());
    let hash_toml = generate_cargo_toml(&hash_modules, &required_crates, "sifr_output");
    assert!(hash_toml.contains("sifr_stdlib = { path = "));
    assert!(hash_toml.contains("default-features = false"));
    assert!(hash_toml.contains("features = [\"hash\"]"));
    assert!(!hash_toml.contains("sha2 = "));
    assert!(!hash_toml.contains("md5 = "));
    let combined_toml = generate_cargo_toml(&combined_modules, &required_crates, "sifr_output");
    assert!(combined_toml.contains(
        "features = [\"html\", \"calendar\", \"platform\", \"uuid\", \"math\", \"hash\"]"
    ));
    assert_eq!(combined_toml.matches("sifr_stdlib = ").count(), 1);
}
