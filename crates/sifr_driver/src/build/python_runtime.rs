use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagePythonRuntime {
    venv_root: PathBuf,
    interpreter: PathBuf,
    executable: String,
    sys_prefix: String,
    sys_base_prefix: String,
    probe_digest: String,
    implementation_name: String,
    implementation_version: String,
    cpython_version_tuple: Vec<u64>,
    sys_path: Vec<String>,
    site_packages: Vec<String>,
}

impl PackagePythonRuntime {
    #[must_use]
    pub fn from_probe(
        request: &sifr_package::PythonEnvironmentProbeRequest,
        probe: &sifr_package::PythonEnvironmentProbe,
        probe_digest: String,
    ) -> Self {
        Self {
            venv_root: request.venv_root.clone(),
            interpreter: request.interpreter.clone(),
            executable: probe.executable.clone(),
            sys_prefix: probe.sys_prefix.clone(),
            sys_base_prefix: probe.sys_base_prefix.clone(),
            probe_digest,
            implementation_name: probe.implementation_name.clone(),
            implementation_version: probe.implementation_version.clone(),
            cpython_version_tuple: probe.cpython_version_tuple.clone(),
            sys_path: probe.sys_path.clone(),
            site_packages: probe.site_packages.clone(),
        }
    }

    #[must_use]
    pub fn probe_digest(&self) -> &str {
        &self.probe_digest
    }

    #[must_use]
    pub(crate) fn interpreter(&self) -> &std::path::Path {
        &self.interpreter
    }

    #[cfg(test)]
    pub(super) fn for_tests(interpreter: &str, probe_digest: &str) -> Self {
        Self {
            venv_root: PathBuf::from("/tmp/sifr-py"),
            interpreter: PathBuf::from(interpreter),
            executable: interpreter.to_string(),
            sys_prefix: "/tmp/sifr-py".to_string(),
            sys_base_prefix: "/opt/python".to_string(),
            probe_digest: probe_digest.to_string(),
            implementation_name: "cpython".to_string(),
            implementation_version: "3.13.1".to_string(),
            cpython_version_tuple: vec![3, 13, 1],
            sys_path: vec!["/tmp/sifr-py/lib".to_string()],
            site_packages: vec!["/tmp/sifr-py/site-packages".to_string()],
        }
    }
}

pub(super) fn render_python_runtime_prelude(metadata: &PackagePythonRuntime) -> String {
    format!(
        r#"fn __sifr_python_runtime_config() -> sifr_runtime::python::PythonRuntimeConfig {{
    sifr_runtime::python::PythonRuntimeConfig {{
        venv_root: {venv_root}.to_string(),
        interpreter: {interpreter}.to_string(),
        executable: {executable}.to_string(),
        sys_prefix: {sys_prefix}.to_string(),
        sys_base_prefix: {sys_base_prefix}.to_string(),
        probe_digest: {probe_digest}.to_string(),
        implementation_name: {implementation_name}.to_string(),
        implementation_version: {implementation_version}.to_string(),
        cpython_version_tuple: vec![{version_tuple}],
        sys_path: vec![{sys_path}],
        site_packages: vec![{site_packages}],
    }}
}}

fn __sifr_initialize_python_runtime() -> Result<(), sifr_runtime::python::PythonRuntimeError> {{
    sifr_runtime::python::initialize_runtime(__sifr_python_runtime_config()).map(|_| ())
}}

"#,
        venv_root = rust_string_literal(&metadata.venv_root.to_string_lossy()),
        interpreter = rust_string_literal(&metadata.interpreter.to_string_lossy()),
        executable = rust_string_literal(&metadata.executable),
        sys_prefix = rust_string_literal(&metadata.sys_prefix),
        sys_base_prefix = rust_string_literal(&metadata.sys_base_prefix),
        probe_digest = rust_string_literal(&metadata.probe_digest),
        implementation_name = rust_string_literal(&metadata.implementation_name),
        implementation_version = rust_string_literal(&metadata.implementation_version),
        version_tuple = render_u64_vec(&metadata.cpython_version_tuple),
        sys_path = render_string_vec(&metadata.sys_path),
        site_packages = render_string_vec(&metadata.site_packages),
    )
}

pub(super) fn inject_python_runtime_bootstrap(
    main_rs: &str,
    metadata: &PackagePythonRuntime,
) -> Result<String, String> {
    let Some(main_start) = main_rs.find("fn main") else {
        return Err(
            "generated package project has Python runtime metadata but no main function"
                .to_string(),
        );
    };
    let Some(body_offset) = main_rs[main_start..].find('{') else {
        return Err("generated package project main function has no body".to_string());
    };
    let insert_at = main_start + body_offset + 1;
    let mut with_bootstrap = render_python_runtime_prelude(metadata);
    with_bootstrap.push_str(&main_rs[..insert_at]);
    with_bootstrap.push_str(
        "\n    if let Err(__sifr_python_runtime_error) = __sifr_initialize_python_runtime() {\n        eprintln!(\"Sifr Python runtime initialization failed: {}\", __sifr_python_runtime_error);\n        std::process::exit(1);\n    }\n",
    );
    with_bootstrap.push_str(&main_rs[insert_at..]);
    Ok(with_bootstrap)
}

fn render_u64_vec(values: &[u64]) -> String {
    values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_string_vec(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("{}.to_string()", rust_string_literal(value)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> PackagePythonRuntime {
        let mut metadata = PackagePythonRuntime::for_tests("/tmp/sifr py/bin/python", "digest-a");
        metadata.venv_root = PathBuf::from("/tmp/sifr py");
        metadata.sys_path = vec!["/tmp/sifr py/lib".to_string()];
        metadata.site_packages = vec!["/tmp/sifr py/site-packages".to_string()];
        metadata
    }

    #[test]
    fn renders_escaped_runtime_metadata() {
        let rendered = render_python_runtime_prelude(&metadata());

        assert!(rendered.contains("PythonRuntimeConfig"));
        assert!(rendered.contains("\"/tmp/sifr py/bin/python\".to_string()"));
        assert!(rendered.contains("cpython_version_tuple: vec![3, 13, 1]"));
    }

    #[test]
    fn injects_runtime_init_as_first_main_statement() {
        let main_rs = "fn main() {\n    println!(\"ok\");\n}\n".to_string();

        let rendered =
            inject_python_runtime_bootstrap(&main_rs, &metadata()).expect("main should be patched");

        assert!(rendered.starts_with("fn __sifr_python_runtime_config()"));
        assert!(rendered.contains("fn main() {\n    if let Err(__sifr_python_runtime_error)"));
        assert!(rendered.contains("println!(\"ok\")"));
    }

    #[test]
    fn rejects_missing_main_function() {
        let error = inject_python_runtime_bootstrap("fn helper() {}\n", &metadata())
            .expect_err("missing main should fail");

        assert!(error.contains("no main function"));
    }
}
