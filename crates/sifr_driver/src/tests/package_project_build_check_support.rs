use std::path::Path;

pub(super) fn local_python_runtime(project_root: &Path) -> crate::PackagePythonRuntime {
    local_python_runtime_with_roots(project_root, &[])
}

pub(super) fn local_python_runtime_with_roots(
    project_root: &Path,
    roots: &[&str],
) -> crate::PackagePythonRuntime {
    let pyproject = project_root.join("pyproject.toml");
    let lock = project_root.join("uv.lock");
    std::fs::write(
        &pyproject,
        "[project]\nname = \"sifr-bridge-test\"\nversion = \"0.0.0\"\nrequires-python = \">=3.11\"\n",
    )
    .expect("test pyproject should be written");
    let uv = std::process::Command::new("uv")
        .args(["lock", "--project"])
        .arg(project_root)
        .output()
        .expect("uv should create the test lock");
    assert!(
        uv.status.success(),
        "uv lock should pass: {}",
        String::from_utf8_lossy(&uv.stderr)
    );
    let venv_root = project_root.join(".venv");
    let uv = std::process::Command::new("uv")
        .args(["venv", "--python"])
        .arg(if cfg!(windows) { "python" } else { "python3" })
        .arg(&venv_root)
        .output()
        .expect("uv should create the test venv");
    assert!(
        uv.status.success(),
        "uv venv should pass: {}",
        String::from_utf8_lossy(&uv.stderr)
    );
    let interpreter = if cfg!(windows) {
        venv_root.join("Scripts/python.exe")
    } else {
        venv_root.join("bin/python")
    };
    let request = sifr_package::PythonEnvironmentProbeRequest {
        venv_root,
        interpreter,
        pyproject: Some(pyproject),
        lock: Some(lock),
        required_imports: Vec::new(),
        declared_imports: Vec::new(),
        native_imports: Vec::new(),
    };
    let probe = sifr_package::probe_python_environment(&request)
        .expect("local CPython environment should probe");
    let digest = sifr_package::digest_python_environment_probe(&request, &probe).hex;
    crate::PackagePythonRuntime::from_probe(
        &request,
        &probe,
        digest,
        roots.iter().map(|root| (*root).to_string()).collect(),
        roots.iter().map(|root| (*root).to_string()).collect(),
        Vec::new(),
    )
}
