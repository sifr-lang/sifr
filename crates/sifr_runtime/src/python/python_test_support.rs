use super::PythonRuntimeConfig;

pub(super) fn local_python_config() -> PythonRuntimeConfig {
    let script = r#"
import os
import sys
print(sys.executable)
print(sys.prefix)
print(sys.base_prefix)
print(".".join(str(part) for part in sys.version_info[:3]))
print(os.pathsep.join(sys.path))
"#;
    let output = std::process::Command::new("python3")
        .args(["-c", script])
        .output()
        .expect("python3 should be available for PyO3 runtime tests");
    assert!(
        output.status.success(),
        "python3 probe should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("python3 probe should emit UTF-8");
    let mut lines = stdout.lines();
    let executable = lines
        .next()
        .expect("python3 probe should emit executable")
        .to_string();
    let sys_prefix = lines
        .next()
        .expect("python3 probe should emit sys.prefix")
        .to_string();
    let sys_base_prefix = lines
        .next()
        .expect("python3 probe should emit sys.base_prefix")
        .to_string();
    let version = lines
        .next()
        .expect("python3 probe should emit version tuple")
        .split('.')
        .map(|part| part.parse::<u64>().expect("version part should be numeric"))
        .collect::<Vec<_>>();
    let sys_path = lines
        .next()
        .unwrap_or_default()
        .split(if cfg!(windows) { ';' } else { ':' })
        .map(str::to_string)
        .collect::<Vec<_>>();
    PythonRuntimeConfig {
        venv_root: sys_prefix.clone(),
        interpreter: executable.clone(),
        executable,
        sys_prefix,
        sys_base_prefix,
        probe_digest: "digest-test".to_string(),
        implementation_name: "cpython".to_string(),
        implementation_version: "test".to_string(),
        cpython_version_tuple: version,
        sys_path,
        site_packages: Vec::new(),
        required_import_roots: vec![
            "asyncio".to_string(),
            "builtins".to_string(),
            "contextlib".to_string(),
            "math".to_string(),
            "sys".to_string(),
        ],
        trusted_import_roots: vec![
            "asyncio".to_string(),
            "builtins".to_string(),
            "contextlib".to_string(),
            "math".to_string(),
            "sys".to_string(),
        ],
        native_import_roots: Vec::new(),
        trusted_native_roots: Vec::new(),
        bridge_sources: Vec::new(),
        start_async_loop: false,
    }
}
