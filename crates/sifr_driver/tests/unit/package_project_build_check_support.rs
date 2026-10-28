use std::path::PathBuf;

pub(super) fn shared_python_runtime() -> crate::PackagePythonRuntime {
    shared_python_runtime_with_roots(&[])
}

pub(super) fn shared_python_runtime_with_roots(roots: &[&str]) -> crate::PackagePythonRuntime {
    // These fixtures exercise generated Python behavior, not environment setup.
    // Keep one locked interpreter while each caller owns fresh source/runtime state.
    let area_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../verification/areas/python_interop");
    let pyproject = area_root.join("pyproject.toml");
    let lock = area_root.join("uv.lock");
    let venv_root = area_root.join(".venv");
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
        .expect("prepare the locked verification/areas/python_interop environment before driver Python tests");
    let digest = sifr_package::digest_python_environment_probe(&request, &probe)
        .expect("probe identity")
        .hex;
    crate::PackagePythonRuntime::from_probe(
        &request,
        &probe,
        digest,
        roots.iter().map(|root| (*root).to_string()).collect(),
        roots.iter().map(|root| (*root).to_string()).collect(),
        Vec::new(),
    )
    .expect("authoring identity")
}
