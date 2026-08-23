use super::*;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn binding_artifact_binds_environment_sources_and_generated_bytes() {
    let root = temp_root("valid");
    fs::create_dir_all(root.join("src")).expect("source directory");
    fs::create_dir_all(root.join("typing")).expect("typing directory");
    fs::write(root.join("src/math_python.sifr"), "def sqrt(): ...\n").expect("binding");
    fs::write(
        root.join("typing/math.pyi"),
        "def sqrt(value: float, /) -> float: ...\n",
    )
    .expect("typing source");
    let artifact = artifact(&root);

    let path = write_python_bindings(&root, &artifact).expect("artifact should write");
    assert_eq!(path, root.join(PYTHON_BINDINGS_FILE));
    assert_eq!(
        load_python_bindings(&root, "environment-a").expect("artifact should load"),
        artifact
    );
    assert_eq!(
        required_python_binding_archive_entries(&root),
        std::collections::BTreeSet::from([
            std::path::PathBuf::from(PYTHON_BINDINGS_FILE),
            std::path::PathBuf::from("src/math_python.sifr"),
            std::path::PathBuf::from("typing/math.pyi"),
        ])
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

#[test]
fn binding_artifact_rejects_environment_generated_and_path_drift() {
    let root = temp_root("drift");
    fs::create_dir_all(root.join("src")).expect("source directory");
    fs::create_dir_all(root.join("typing")).expect("typing directory");
    fs::write(root.join("src/math_python.sifr"), "def sqrt(): ...\n").expect("binding");
    fs::write(root.join("typing/math.pyi"), "def sqrt() -> float: ...\n").expect("typing source");
    let artifact = artifact(&root);
    write_python_bindings(&root, &artifact).expect("artifact should write");

    assert!(
        load_python_bindings(&root, "environment-b")
            .expect_err("environment drift must fail")
            .contains("environment digest")
    );
    fs::write(
        root.join("typing/math.pyi"),
        "def sqrt(value: int) -> int: ...\n",
    )
    .expect("mutate typing source");
    assert!(
        load_python_bindings(&root, "environment-a")
            .expect_err("typing source drift must fail")
            .contains("typing source")
    );
    fs::write(root.join("typing/math.pyi"), "def sqrt() -> float: ...\n")
        .expect("restore typing source");
    fs::write(root.join("src/math_python.sifr"), "def changed(): ...\n").expect("mutate output");
    assert!(
        load_python_bindings(&root, "environment-a")
            .expect_err("generated drift must fail")
            .contains("has drifted")
    );

    let mut escaping = artifact;
    escaping.bindings[0].output = "../escape.sifr".to_string();
    assert!(
        validate_python_bindings(&root, "environment-a", &escaping)
            .expect_err("escaping output must fail")
            .contains("stay inside")
    );
    fs::remove_dir_all(root).expect("remove fixture");
}

#[cfg(unix)]
#[test]
fn binding_artifact_rejects_symlinked_typing_sources_and_outputs() {
    use std::os::unix::fs::symlink;

    let root = temp_root("symlink");
    let outside = temp_root("symlink-outside");
    fs::create_dir_all(root.join("src")).expect("source directory");
    fs::create_dir_all(root.join("typing")).expect("typing directory");
    fs::create_dir_all(&outside).expect("outside directory");
    let generated = "def sqrt(): ...\n";
    let typing = "def sqrt() -> float: ...\n";
    fs::write(root.join("src/math_python.sifr"), generated).expect("binding");
    fs::write(root.join("typing/math.pyi"), typing).expect("typing source");
    fs::write(outside.join("math.pyi"), typing).expect("outside typing source");
    fs::write(outside.join("math_python.sifr"), generated).expect("outside binding");
    let artifact = artifact(&root);
    write_python_bindings(&root, &artifact).expect("artifact should write");

    fs::remove_file(root.join("typing/math.pyi")).expect("remove typing source");
    symlink(outside.join("math.pyi"), root.join("typing/math.pyi")).expect("typing symlink");
    let typing_error = load_python_bindings(&root, "environment-a")
        .expect_err("symlinked typing source must fail");
    assert!(
        typing_error.contains("regular package file"),
        "{typing_error}"
    );

    fs::remove_file(root.join("typing/math.pyi")).expect("remove typing symlink");
    fs::write(root.join("typing/math.pyi"), typing).expect("restore typing source");
    fs::remove_file(root.join("src/math_python.sifr")).expect("remove generated source");
    symlink(
        outside.join("math_python.sifr"),
        root.join("src/math_python.sifr"),
    )
    .expect("output symlink");
    let output_error =
        load_python_bindings(&root, "environment-a").expect_err("symlinked output must fail");
    assert!(
        output_error.contains("regular package file"),
        "{output_error}"
    );

    fs::remove_dir_all(root).expect("remove fixture");
    fs::remove_dir_all(outside).expect("remove outside fixture");
}

#[test]
fn binding_output_rejects_managed_metadata_files() {
    let root = temp_root("reserved-output");
    for reserved in [PYTHON_BINDINGS_FILE, super::PYTHON_CERTIFICATIONS_FILE] {
        let error = safe_python_binding_output(&root, Path::new(reserved))
            .expect_err("managed metadata output must be rejected");
        assert!(error.contains("reserved for package metadata"), "{error}");
    }
}

fn artifact(root: &std::path::Path) -> PythonBindingArtifact {
    let symbols = vec!["sqrt".to_string()];
    let sources = vec![PythonBindingSource {
        symbol: "sqrt".to_string(),
        kind: PythonBindingSourceKind::Override,
        identity: "override:0:math.pyi".to_string(),
        digest: python_binding_generated_digest(
            &fs::read(root.join("typing/math.pyi")).expect("typing source"),
        ),
    }];
    let soabi = "cpython-test";
    let source_fingerprint =
        python_binding_source_fingerprint("math", &symbols, soabi, None, &sources);
    let generated = fs::read(root.join("src/math_python.sifr")).expect("generated source");
    PythonBindingArtifact {
        schema_version: PYTHON_BINDING_SCHEMA_VERSION,
        environment_digest: "environment-a".to_string(),
        bindings: vec![PythonBinding {
            module: "math".to_string(),
            symbols,
            output: "src/math_python.sifr".to_string(),
            soabi: soabi.to_string(),
            distribution: None,
            overrides: vec!["typing/math.pyi".to_string()],
            stub_packages: Vec::new(),
            external_stubs: Vec::new(),
            sources,
            source_fingerprint,
            generated_digest: python_binding_generated_digest(&generated),
        }],
    }
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "sifr-python-binding-{label}-{}-{nonce}",
        std::process::id()
    ))
}
