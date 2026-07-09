use super::*;

#[test]
pub(crate) fn test_generate_cargo_toml_stateless_sysroot_modules_enable_stdlib_features() {
    let combined_modules = normalize_dependency_set(
        [
            "sifr.platform",
            "sifr.env",
            "sifr.sys",
            "sifr.html",
            "_sifr.calendar",
            "sifr.uuid",
            "sifr.collections",
            "sifr.math",
            "sifr.hashlib",
            "sifr.base64",
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
    let collections_modules =
        normalize_dependency_set(vec!["sifr.collections".to_string()].into_iter());
    let collections_toml =
        generate_cargo_toml(&collections_modules, &required_crates, "sifr_output");
    assert!(collections_toml.contains("sifr_stdlib = { path = "));
    assert!(collections_toml.contains("features = [\"collections\"]"));
    assert!(!collections_toml.contains("serde_json = "));
    assert!(!collections_toml.contains("serde = "));
    let hash_modules = normalize_dependency_set(vec!["sifr.hashlib".to_string()].into_iter());
    let hash_toml = generate_cargo_toml(&hash_modules, &required_crates, "sifr_output");
    assert!(hash_toml.contains("sifr_stdlib = { path = "));
    assert!(hash_toml.contains("default-features = false"));
    assert!(hash_toml.contains("features = [\"hash\", \"bytes\"]"));
    assert!(!hash_toml.contains("sha2 = "));
    assert!(!hash_toml.contains("md5 = "));
    let base64_modules = normalize_dependency_set(vec!["sifr.base64".to_string()].into_iter());
    let base64_toml = generate_cargo_toml(&base64_modules, &required_crates, "sifr_output");
    assert!(base64_toml.contains("sifr_stdlib = { path = "));
    assert!(base64_toml.contains("default-features = false"));
    assert!(base64_toml.contains("features = [\"base64\", \"bytes\"]"));
    assert!(base64_toml.contains("base64 = \"0.22.1\""));
    assert!(!base64_toml.contains("bytes = \""));
    let random_modules = normalize_dependency_set(vec!["sifr.random".to_string()].into_iter());
    let random_toml = generate_cargo_toml(&random_modules, &required_crates, "sifr_output");
    assert!(random_toml.contains("sifr_stdlib = { path = "));
    assert!(random_toml.contains("features = [\"random\"]"));
    let logging_modules = normalize_dependency_set(vec!["sifr.logging".to_string()].into_iter());
    let logging_toml = generate_cargo_toml(&logging_modules, &required_crates, "sifr_output");
    assert!(logging_toml.contains("sifr_stdlib = { path = "));
    assert!(logging_toml.contains("features = [\"logging\"]"));
    let time_modules = normalize_dependency_set(vec!["sifr.time".to_string()].into_iter());
    let time_toml = generate_cargo_toml(&time_modules, &required_crates, "sifr_output");
    assert!(time_toml.contains("sifr_stdlib = { path = "));
    assert!(time_toml.contains("features = [\"time\"]"));
    assert!(!time_toml.contains("chrono = "));
    let sys_modules = normalize_dependency_set(vec!["sifr.sys".to_string()].into_iter());
    let sys_toml = generate_cargo_toml(&sys_modules, &required_crates, "sifr_output");
    assert!(sys_toml.contains("sifr_stdlib = { path = "));
    assert!(sys_toml.contains("features = [\"sys\"]"));
    let private_crypto_modules =
        normalize_dependency_set(vec!["_sifr.crypto".to_string()].into_iter());
    let private_crypto_toml =
        generate_cargo_toml(&private_crypto_modules, &required_crates, "sifr_output");
    assert!(private_crypto_toml.contains("sifr_stdlib = { path = "));
    assert!(private_crypto_toml.contains("features = [\"random\", \"hash\", \"base64\"]"));
    let bytes_modules = normalize_dependency_set(vec!["sifr.bytes".to_string()].into_iter());
    let bytes_toml = generate_cargo_toml(&bytes_modules, &required_crates, "sifr_output");
    assert!(bytes_toml.contains("sifr_stdlib = { path = "));
    assert!(bytes_toml.contains("features = [\"bytes\"]"));
    assert!(!bytes_toml.contains("bytes = \""));
    let datetime_modules = normalize_dependency_set(vec!["sifr.datetime".to_string()].into_iter());
    let datetime_toml = generate_cargo_toml(&datetime_modules, &required_crates, "sifr_output");
    assert!(datetime_toml.contains("sifr_stdlib = { path = "));
    assert!(datetime_toml.contains("features = [\"time\"]"));
    assert!(!datetime_toml.contains("chrono = "));
    let gzip_modules = normalize_dependency_set(vec!["sifr.gzip".to_string()].into_iter());
    let gzip_toml = generate_cargo_toml(&gzip_modules, &required_crates, "sifr_output");
    assert!(gzip_toml.contains("sifr_stdlib = { path = "));
    assert!(gzip_toml.contains("features = [\"gzip\"]"));
    assert!(!gzip_toml.contains("flate2 = "));
    let zip_modules = normalize_dependency_set(vec!["sifr.zipfile".to_string()].into_iter());
    let zip_toml = generate_cargo_toml(&zip_modules, &required_crates, "sifr_output");
    assert!(zip_toml.contains("sifr_stdlib = { path = "));
    assert!(zip_toml.contains("features = [\"zipfile\"]"));
    assert!(!zip_toml.contains("zip = "));
    let fs_modules = normalize_dependency_set(vec!["_sifr.fs".to_string()].into_iter());
    let fs_toml = generate_cargo_toml(&fs_modules, &required_crates, "sifr_output");
    assert!(fs_toml.contains("sifr_stdlib = { path = "));
    assert!(fs_toml.contains("features = [\"fs\"]"));
    let private_compress_modules =
        normalize_dependency_set(vec!["_sifr.compress".to_string()].into_iter());
    let private_compress_toml =
        generate_cargo_toml(&private_compress_modules, &required_crates, "sifr_output");
    assert!(private_compress_toml.contains("sifr_stdlib = { path = "));
    assert!(private_compress_toml.contains("features = [\"gzip\", \"zipfile\"]"));
    let process_modules = normalize_dependency_set(vec!["sifr.process".to_string()].into_iter());
    let process_toml = generate_cargo_toml(&process_modules, &required_crates, "sifr_output");
    assert!(process_toml.contains("sifr_stdlib = { path = "));
    assert!(process_toml.contains("features = [\"process\"]"));
    let private_process_modules =
        normalize_dependency_set(vec!["_sifr.process".to_string()].into_iter());
    let private_process_toml =
        generate_cargo_toml(&private_process_modules, &required_crates, "sifr_output");
    assert!(private_process_toml.contains("sifr_stdlib = { path = "));
    assert!(private_process_toml.contains("features = [\"process\"]"));
    let combined_toml = generate_cargo_toml(&combined_modules, &required_crates, "sifr_output");
    assert!(combined_toml.contains(
        "features = [\"html\", \"calendar\", \"platform\", \"sys\", \"uuid\", \"collections\", \"math\", \"hash\", \"base64\", \"bytes\"]"
    ));
    assert_eq!(combined_toml.matches("sifr_stdlib = ").count(), 1);
}
