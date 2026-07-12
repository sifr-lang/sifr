use std::path::PathBuf;

use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::{LoweringOptions, PythonTrustPolicy};

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
    required_import_roots: Vec<String>,
    trusted_import_roots: Vec<String>,
    native_import_roots: Vec<String>,
    trusted_native_roots: Vec<String>,
    libpython: Option<String>,
    bridge_sources: Vec<EmbeddedPythonBridgeSource>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EmbeddedPythonBridgeSource {
    pub module: String,
    pub source: String,
    pub filename: String,
    pub is_package: bool,
    pub package_prefix: String,
}

impl PackagePythonRuntime {
    #[must_use]
    pub fn from_probe(
        request: &sifr_package::PythonEnvironmentProbeRequest,
        probe: &sifr_package::PythonEnvironmentProbe,
        probe_digest: String,
        required_import_roots: Vec<String>,
        trusted_import_roots: Vec<String>,
        trusted_native_roots: Vec<String>,
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
            required_import_roots,
            trusted_import_roots,
            native_import_roots: detected_native_import_roots(probe),
            trusted_native_roots,
            libpython: probe.libpython.clone(),
            bridge_sources: Vec::new(),
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

    #[must_use]
    pub(super) fn lowering_options(&self) -> LoweringOptions {
        LoweringOptions {
            python_trust_policy: Some(PythonTrustPolicy {
                required_import_roots: self.required_import_roots.clone(),
                trusted_import_roots: self.trusted_import_roots.clone(),
            }),
            ..LoweringOptions::default()
        }
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
            required_import_roots: vec!["math".to_string()],
            trusted_import_roots: vec!["math".to_string()],
            native_import_roots: Vec::new(),
            trusted_native_roots: Vec::new(),
            libpython: None,
            bridge_sources: Vec::new(),
        }
    }

    #[must_use]
    pub(super) fn trusted_native_link_names(&self) -> Vec<String> {
        let mut links = Vec::new();
        if let Some(libpython) = &self.libpython {
            if let Some(link_name) = native_link_name_from_libpython(libpython) {
                links.push(link_name);
            }
        }
        if self.implementation_name.eq_ignore_ascii_case("cpython")
            && self.cpython_version_tuple.len() >= 2
        {
            links.push(format!(
                "python{}.{}",
                self.cpython_version_tuple[0], self.cpython_version_tuple[1]
            ));
        }
        links.sort();
        links.dedup();
        links
    }

    #[cfg(test)]
    pub(super) fn set_libpython_for_tests(&mut self, libpython: &str) {
        self.libpython = Some(libpython.to_string());
    }

    pub(super) fn set_bridge_sources(&mut self, sources: Vec<EmbeddedPythonBridgeSource>) {
        self.bridge_sources = sources;
    }
}

fn native_link_name_from_libpython(libpython: &str) -> Option<String> {
    if let Some((_, suffix)) = libpython.split_once("Python.framework/Versions/") {
        if let Some((version, _)) = suffix.split_once('/') {
            if !version.is_empty() {
                return Some(format!("python{version}"));
            }
        }
    }
    let file_name = std::path::Path::new(libpython).file_name()?.to_str()?;
    let without_prefix = file_name.strip_prefix("lib").unwrap_or(file_name);
    let without_suffix = without_prefix
        .strip_suffix(".dylib")
        .or_else(|| without_prefix.strip_suffix(".a"))
        .or_else(|| without_prefix.split_once(".so").map(|(name, _)| name))
        .unwrap_or(without_prefix);
    if without_suffix.is_empty() {
        None
    } else {
        Some(without_suffix.to_string())
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
        required_import_roots: vec![{required_import_roots}],
        trusted_import_roots: vec![{trusted_import_roots}],
        native_import_roots: vec![{native_import_roots}],
        trusted_native_roots: vec![{trusted_native_roots}],
        bridge_sources: vec![{bridge_sources}],
    }}
}}

fn __sifr_initialize_python_runtime() -> Result<sifr_runtime::python::PythonRuntimeGuard, sifr_runtime::python::PythonRuntimeError> {{
    sifr_runtime::python::initialize_runtime(__sifr_python_runtime_config())?;
    sifr_runtime::python::runtime_guard()
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
        required_import_roots = render_string_vec(&metadata.required_import_roots),
        trusted_import_roots = render_string_vec(&metadata.trusted_import_roots),
        native_import_roots = render_string_vec(&metadata.native_import_roots),
        trusted_native_roots = render_string_vec(&metadata.trusted_native_roots),
        bridge_sources = render_bridge_sources(&metadata.bridge_sources),
    )
}

pub(super) fn inject_python_runtime_bootstrap(
    main_rs: &str,
    metadata: &PackagePythonRuntime,
) -> Result<String, String> {
    let insert_at = find_main_body_insert(main_rs).ok_or_else(|| {
        "generated package project has Python runtime metadata but no main function".to_string()
    })?;
    let mut with_bootstrap = render_python_runtime_prelude(metadata);
    with_bootstrap.push_str(&main_rs[..insert_at]);
    with_bootstrap.push_str(&format!(
        "\n    let __sifr_python_runtime_guard = match __sifr_initialize_python_runtime() {{\n        Ok(__sifr_python_runtime_guard) => __sifr_python_runtime_guard,\n        Err(sifr_runtime::python::PythonRuntimeError::ReservedBridgeCollision {{ module }}) => {{\n            eprintln!(\"{collision_code}: reserved Python bridge namespace collision at '{{}}'\", module);\n            std::process::exit(1);\n        }}\n        Err(__sifr_python_runtime_error) => {{\n            eprintln!(\"Sifr Python runtime initialization failed: {{}}\", __sifr_python_runtime_error);\n            std::process::exit(1);\n        }}\n    }};\n",
        collision_code = DiagnosticCode::PYIMP_RESERVED_BRIDGE_COLLISION.code(),
    ));
    with_bootstrap.push_str(&main_rs[insert_at..]);
    Ok(with_bootstrap)
}

fn find_main_body_insert(main_rs: &str) -> Option<usize> {
    find_function_body_insert(main_rs, "fn main(")
        .or_else(|| find_function_body_insert(main_rs, "async fn main("))
}

fn find_function_body_insert(source: &str, signature_start: &str) -> Option<usize> {
    let main_start = source
        .find(&format!("\n{signature_start}"))
        .map(|index| index + 1)
        .or_else(|| source.strip_prefix(signature_start).map(|_| 0))?;
    let body_offset = source[main_start..].find('{')?;
    Some(main_start + body_offset + 1)
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

fn render_bridge_sources(values: &[EmbeddedPythonBridgeSource]) -> String {
    values
        .iter()
        .map(|value| {
            format!(
                "sifr_runtime::python::PythonBridgeSource {{ module: {}.to_string(), source: {}.to_string(), filename: {}.to_string(), is_package: {}, package_prefix: {}.to_string() }}",
                rust_string_literal(&value.module),
                rust_string_literal(&value.source),
                rust_string_literal(&value.filename),
                value.is_package,
                rust_string_literal(&value.package_prefix),
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn rust_string_literal(value: &str) -> String {
    format!("{value:?}")
}

fn detected_native_import_roots(probe: &sifr_package::PythonEnvironmentProbe) -> Vec<String> {
    probe
        .imports
        .iter()
        .chain(probe.native_imports.iter())
        .filter(|import| import.ok)
        .filter(|import| {
            import.origin.as_ref().is_some_and(|origin| {
                probe
                    .extension_suffixes
                    .iter()
                    .any(|suffix| origin.ends_with(suffix))
            })
        })
        .map(|import| import.root.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
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
    fn native_import_roots_are_detected_from_probe_origins() {
        let request = sifr_package::PythonEnvironmentProbeRequest {
            venv_root: PathBuf::from("/tmp/sifr-py"),
            interpreter: PathBuf::from("/tmp/sifr-py/bin/python"),
            pyproject: None,
            lock: None,
            required_imports: Vec::new(),
            declared_imports: vec!["numpy".to_string()],
            native_imports: Vec::new(),
        };
        let probe = sifr_package::PythonEnvironmentProbe {
            implementation_name: "CPython".to_string(),
            implementation_version: "3.13.1".to_string(),
            cpython_version_tuple: vec![3, 13, 1],
            executable: "/tmp/sifr-py/bin/python".to_string(),
            sys_prefix: "/tmp/sifr-py".to_string(),
            sys_base_prefix: "/opt/python".to_string(),
            site_packages: vec!["/tmp/sifr-py/site-packages".to_string()],
            sys_path: vec!["/tmp/sifr-py/lib".to_string()],
            soabi: Some("cpython-313-darwin".to_string()),
            extension_suffixes: vec![".cpython-313-darwin.so".to_string()],
            pointer_width: 64,
            platform: "macOS".to_string(),
            machine: "arm64".to_string(),
            libpython: None,
            free_threaded: false,
            imports: vec![sifr_package::python::PythonImportProbe {
                root: "numpy".to_string(),
                ok: true,
                origin: Some(
                    "/tmp/sifr-py/site-packages/numpy/_core.cpython-313-darwin.so".to_string(),
                ),
                error: None,
            }],
            native_imports: Vec::new(),
            pyproject_digest: None,
            uv_lock_digest: None,
        };
        let metadata = PackagePythonRuntime::from_probe(
            &request,
            &probe,
            "digest".to_string(),
            vec!["numpy".to_string()],
            vec!["numpy".to_string()],
            Vec::new(),
        );
        let rendered = render_python_runtime_prelude(&metadata);

        assert!(rendered.contains("native_import_roots: vec![\"numpy\".to_string()]"));
        assert!(rendered.contains("trusted_native_roots: vec![]"));
    }

    #[test]
    fn trusted_native_link_names_include_selected_libpython() {
        let mut metadata = metadata();
        metadata.set_libpython_for_tests("/opt/python/lib/libpython3.13.dylib");

        assert_eq!(metadata.trusted_native_link_names(), ["python3.13"]);
    }

    #[test]
    fn trusted_native_link_names_include_cpython_framework_version() {
        let mut metadata = metadata();
        metadata.set_libpython_for_tests(
            "/opt/homebrew/Frameworks/Python.framework/Versions/3.13/Python",
        );

        assert_eq!(metadata.trusted_native_link_names(), ["python3.13"]);
    }

    #[test]
    fn trusted_native_link_names_fall_back_to_cpython_version() {
        let metadata = metadata();

        assert_eq!(metadata.trusted_native_link_names(), ["python3.13"]);
    }

    #[test]
    fn injects_runtime_init_as_first_main_statement() {
        let main_rs = "fn main() {\n    println!(\"ok\");\n}\n".to_string();

        let rendered =
            inject_python_runtime_bootstrap(&main_rs, &metadata()).expect("main should be patched");

        assert!(rendered.starts_with("fn __sifr_python_runtime_config()"));
        assert!(rendered.contains(
            "fn main() {\n    let __sifr_python_runtime_guard = match __sifr_initialize_python_runtime()"
        ));
        assert!(rendered.contains("SIFR-PYIMP-0003: reserved Python bridge namespace collision"));
        assert!(rendered.contains("println!(\"ok\")"));
    }

    #[test]
    fn injects_runtime_init_into_result_returning_main() {
        let main_rs =
            "fn main() -> Result<(), PythonError> {\n    run_example()?;\n    Ok(())\n}\n"
                .to_string();

        let rendered =
            inject_python_runtime_bootstrap(&main_rs, &metadata()).expect("main should be patched");

        assert!(rendered.contains(
            "fn main() -> Result<(), PythonError> {\n    let __sifr_python_runtime_guard = match __sifr_initialize_python_runtime()"
        ));
        assert!(rendered.contains("run_example()?"));
    }

    #[test]
    fn injects_runtime_init_into_async_main() {
        let main_rs =
            "#[tokio::main]\nasync fn main() -> Result<(), Error> {\n    run().await\n}\n"
                .to_string();

        let rendered =
            inject_python_runtime_bootstrap(&main_rs, &metadata()).expect("main should be patched");

        assert!(rendered.contains(
            "async fn main() -> Result<(), Error> {\n    let __sifr_python_runtime_guard = match __sifr_initialize_python_runtime()"
        ));
        assert!(rendered.contains("run().await"));
    }

    #[test]
    fn rejects_missing_main_function() {
        let error = inject_python_runtime_bootstrap("fn helper() {}\n", &metadata())
            .expect_err("missing main should fail");

        assert!(error.contains("no main function"));
    }
}
