use super::digest::{digest_serializable, GraphDigest};
use crate::graph::derive::SifrPackageGraph;
use serde::Serialize;

#[must_use]
pub fn digest_package_graph(graph: &SifrPackageGraph) -> GraphDigest {
    let canonical = CanonicalGraph::from(graph);
    digest_serializable(&canonical)
}

#[derive(Serialize)]
struct CanonicalGraph<'a> {
    packages: Vec<CanonicalGraphPackage<'a>>,
    edges: Vec<(&'a str, Vec<&'a str>)>,
    backend_crates: Vec<CanonicalBackendCrate<'a>>,
    scopes: Vec<CanonicalScope<'a>>,
}

#[derive(Serialize)]
struct CanonicalGraphPackage<'a> {
    package_id: &'a str,
    cargo_package_id: &'a str,
    sifr_name: &'a str,
    exports: Vec<&'a str>,
    rust: CanonicalRustInteropConfig,
    rust_trust: CanonicalRustTrust<'a>,
    python: CanonicalPythonConfig<'a>,
    python_trust: CanonicalPythonTrust<'a>,
}

#[derive(Serialize)]
struct CanonicalRustInteropConfig {
    bridge_version: Option<u32>,
    bridges: Vec<String>,
    direct_crate_bindings: bool,
}

#[derive(Serialize)]
struct CanonicalRustTrust<'a> {
    rust_build_scripts: Vec<&'a str>,
    rust_proc_macros: Vec<&'a str>,
    native_links: Vec<&'a str>,
    unsafe_rust_bridges: Vec<&'a str>,
    build_env: Vec<&'a str>,
    rust_no_panic: Vec<&'a str>,
    rust_panic_abort: Vec<&'a str>,
}

#[derive(Serialize)]
struct CanonicalPythonConfig<'a> {
    venv: Option<String>,
    pyproject: Option<String>,
    lock: Option<String>,
    interpreter: Option<String>,
    requires_imports: Vec<&'a str>,
}

#[derive(Serialize)]
struct CanonicalPythonTrust<'a> {
    python: Vec<&'a str>,
    python_native: Vec<&'a str>,
}

#[derive(Serialize)]
struct CanonicalBackendCrate<'a> {
    package_id: &'a str,
    dependency_name: &'a str,
    dependency_kind: Option<&'a str>,
    cargo_package_id: &'a str,
    cargo_package_name: &'a str,
    cargo_version: &'a str,
    cargo_source: Option<&'a str>,
    cargo_manifest_path: String,
    links: Option<&'a str>,
    has_build_script: bool,
    has_proc_macro: bool,
}

#[derive(Serialize)]
struct CanonicalScope<'a> {
    package_id: &'a str,
    imports: Vec<(&'a str, &'a str)>,
}

impl<'a> From<&'a SifrPackageGraph> for CanonicalGraph<'a> {
    fn from(graph: &'a SifrPackageGraph) -> Self {
        Self {
            packages: graph
                .packages
                .values()
                .map(|package| CanonicalGraphPackage {
                    package_id: &package.package_id.0,
                    cargo_package_id: &package.cargo_package_id.0,
                    sifr_name: &package.sifr_name.0,
                    exports: package
                        .manifest
                        .exports
                        .iter()
                        .map(|root| root.0.as_str())
                        .collect(),
                    rust: CanonicalRustInteropConfig {
                        bridge_version: package.manifest.rust.bridge_version,
                        bridges: package
                            .manifest
                            .rust
                            .bridges
                            .iter()
                            .map(|path| normalized_path_string(path))
                            .collect(),
                        direct_crate_bindings: package.manifest.rust.direct_crate_bindings,
                    },
                    rust_trust: CanonicalRustTrust {
                        rust_build_scripts: package
                            .manifest
                            .trust
                            .rust_build_scripts
                            .iter()
                            .map(String::as_str)
                            .collect(),
                        rust_proc_macros: package
                            .manifest
                            .trust
                            .rust_proc_macros
                            .iter()
                            .map(String::as_str)
                            .collect(),
                        native_links: package
                            .manifest
                            .trust
                            .native_links
                            .iter()
                            .map(String::as_str)
                            .collect(),
                        unsafe_rust_bridges: package
                            .manifest
                            .trust
                            .unsafe_rust_bridges
                            .iter()
                            .map(String::as_str)
                            .collect(),
                        build_env: package
                            .manifest
                            .trust
                            .build_env
                            .iter()
                            .map(String::as_str)
                            .collect(),
                        rust_no_panic: package
                            .manifest
                            .trust
                            .rust_no_panic
                            .iter()
                            .map(String::as_str)
                            .collect(),
                        rust_panic_abort: package
                            .manifest
                            .trust
                            .rust_panic_abort
                            .iter()
                            .map(String::as_str)
                            .collect(),
                    },
                    python: CanonicalPythonConfig {
                        venv: package
                            .manifest
                            .python
                            .venv
                            .as_ref()
                            .map(|path| path.display().to_string()),
                        pyproject: package
                            .manifest
                            .python
                            .pyproject
                            .as_ref()
                            .map(|path| path.display().to_string()),
                        lock: package
                            .manifest
                            .python
                            .lock
                            .as_ref()
                            .map(|path| path.display().to_string()),
                        interpreter: package
                            .manifest
                            .python
                            .interpreter
                            .as_ref()
                            .map(|path| path.display().to_string()),
                        requires_imports: package
                            .manifest
                            .python
                            .requires_imports
                            .iter()
                            .map(String::as_str)
                            .collect(),
                    },
                    python_trust: CanonicalPythonTrust {
                        python: package
                            .manifest
                            .trust
                            .python
                            .iter()
                            .map(String::as_str)
                            .collect(),
                        python_native: package
                            .manifest
                            .trust
                            .python_native
                            .iter()
                            .map(String::as_str)
                            .collect(),
                    },
                })
                .collect(),
            edges: graph
                .cargo_edges
                .iter()
                .map(|(from, to)| {
                    (
                        from.0.as_str(),
                        to.iter().map(|package_id| package_id.0.as_str()).collect(),
                    )
                })
                .collect(),
            backend_crates: graph
                .backend_crates
                .iter()
                .flat_map(|(package_id, backends)| {
                    backends.iter().map(move |backend| CanonicalBackendCrate {
                        package_id: package_id.0.as_str(),
                        dependency_name: backend.dependency_name.as_str(),
                        dependency_kind: backend.dependency_kind.as_deref(),
                        cargo_package_id: backend.cargo_package_id.0.as_str(),
                        cargo_package_name: backend.cargo_package_name.as_str(),
                        cargo_version: backend.cargo_version.as_str(),
                        cargo_source: backend.cargo_source.as_deref(),
                        cargo_manifest_path: normalized_path_string(&backend.cargo_manifest_path),
                        links: backend.links.as_deref(),
                        has_build_script: backend.has_build_script,
                        has_proc_macro: backend.has_proc_macro,
                    })
                })
                .collect(),
            scopes: graph
                .direct_dependency_scopes
                .iter()
                .map(|(package_id, scope)| CanonicalScope {
                    package_id: &package_id.0,
                    imports: scope
                        .imports
                        .iter()
                        .map(|(root, import)| (root.0.as_str(), import.package_id.0.as_str()))
                        .collect(),
                })
                .collect(),
        }
    }
}

fn normalized_path_string(path: &std::path::Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
