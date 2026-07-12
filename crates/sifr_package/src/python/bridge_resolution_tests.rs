use super::bridge_resolution::{
    resolve_python_bridge_graph, resolved_python_bridge_package_key,
    resolved_python_bridge_runtime_package, ResolvedPythonBridgeImport,
};
use super::requirements::canonical_python_requirements;
use super::test_support::{graph, package, package_id};
use super::trust_policy::validate_python_trust_policy;
use crate::graph::scopes::{DirectDependencyScope, ScopedImport, ScopedImportSource};
use crate::manifest::sifr::{ImportRoot, PythonConfig, TrustPolicy};
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn resolved_package_key_is_stable_distinct_and_a_valid_identifier_segment() {
    let first = package_id("alpha-name");
    let second = package_id("alpha_name");

    let key = resolved_python_bridge_package_key(&first);

    assert_eq!(key, resolved_python_bridge_package_key(&first));
    assert_ne!(key, resolved_python_bridge_package_key(&second));
    assert_eq!(key.len(), 64);
    assert!(key.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let runtime_package = resolved_python_bridge_runtime_package(&first);
    assert!(runtime_package.starts_with("__sifr_bridge__.p_"));
    assert!(runtime_package
        .split('.')
        .all(|component| component.starts_with('_') || component.starts_with('p')));
}

#[test]
fn missing_root_package_is_a_structured_graph_error() {
    let diagnostics = resolve_python_bridge_graph(&graph(Vec::new()), &package_id("missing"))
        .expect_err("missing bridge root package must fail");

    assert_eq!(diagnostics[0].code, DiagnosticCode::PACKAGE_METADATA_PARSE);
    assert!(diagnostics[0]
        .message
        .contains("missing from the package graph"));
}

#[test]
fn selected_bridge_graph_rewrites_package_edges_and_derives_external_requirements() {
    let fixture = ResolutionFixture::new("selected");
    let app = fixture.bridge_package("app", "requests.sessions");
    let dependency = fixture.bridge_package("dependency", "numpy.linalg");
    let unselected = fixture.bridge_package("unselected", "pandas.core");
    let app_id = app.package_id.clone();
    let dependency_id = dependency.package_id.clone();
    let dependency_cargo_id = dependency.cargo_package_id.clone();
    let mut graph = graph(vec![app, dependency, unselected]);
    graph.direct_dependency_scopes.insert(
        app_id.clone(),
        DirectDependencyScope {
            imports: BTreeMap::from([(
                ImportRoot("dependency".to_string()),
                ScopedImport {
                    import_root: ImportRoot("dependency".to_string()),
                    target_export_root: ImportRoot("dependency".to_string()),
                    package_id: dependency_id.clone(),
                    cargo_package_id: dependency_cargo_id,
                    dependency_name: "dependency".to_string(),
                    source: ScopedImportSource::Export,
                },
            )]),
        },
    );

    let resolved = resolve_python_bridge_graph(&graph, &app_id).expect("resolve bridge graph");

    assert_eq!(resolved.packages.len(), 2);
    assert_eq!(
        resolved
            .requirements
            .iter()
            .map(|requirement| requirement.root.as_str())
            .collect::<Vec<_>>(),
        ["numpy", "requests"]
    );
    assert!(!resolved
        .requirements
        .iter()
        .any(|requirement| requirement.root == "pandas"));
    let app = resolved
        .packages
        .iter()
        .find(|package| package.package_id == app_id)
        .expect("app bridge package");
    let dependency = resolved
        .packages
        .iter()
        .find(|package| package.package_id == dependency_id)
        .expect("dependency bridge package");
    assert_ne!(app.runtime_package, dependency.runtime_package);
    for package in [app, dependency] {
        let adapter = package
            .modules
            .iter()
            .find(|module| module.module == "adapter")
            .expect("adapter module");
        assert!(adapter.imports.iter().any(|import| matches!(
            import,
            ResolvedPythonBridgeImport::SamePackage { module, runtime_module }
                if module == "shared"
                    && runtime_module == &format!("{}.shared", package.runtime_package)
        )));
    }
}

#[test]
fn unresolved_same_package_bridge_import_is_rejected() {
    let fixture = ResolutionFixture::new("missing");
    let mut app = fixture.package("app");
    fixture.write(&app, "adapter.py", "import bridge.missing\n");
    let app_id = app.package_id.clone();
    app.manifest.trust.python = vec!["requests".to_string()];

    let diagnostics = resolve_python_bridge_graph(&graph(vec![app]), &app_id)
        .expect_err("missing same-package module must fail");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PYIMP_INVALID_BRIDGE_SOURCE
    );
    assert!(diagnostics[0].message.contains("does not resolve"));
}

#[test]
fn dependency_bridge_requirements_remain_root_authorized() {
    let fixture = ResolutionFixture::new("authority");
    let mut app = fixture.package("app");
    app.manifest.trust.python = vec!["requests".to_string()];
    let dependency = fixture.package("dependency");
    fixture.write(&dependency, "adapter.py", "import numpy.linalg\n");
    let app_id = app.package_id.clone();
    let dependency_id = dependency.package_id.clone();
    let dependency_cargo_id = dependency.cargo_package_id.clone();
    let mut graph = graph(vec![app, dependency]);
    graph.direct_dependency_scopes.insert(
        app_id.clone(),
        DirectDependencyScope {
            imports: BTreeMap::from([(
                ImportRoot("dependency".to_string()),
                ScopedImport {
                    import_root: ImportRoot("dependency".to_string()),
                    target_export_root: ImportRoot("dependency".to_string()),
                    package_id: dependency_id,
                    cargo_package_id: dependency_cargo_id,
                    dependency_name: "dependency".to_string(),
                    source: ScopedImportSource::Export,
                },
            )]),
        },
    );
    let resolved = resolve_python_bridge_graph(&graph, &app_id).expect("resolve requirements");
    let requirements = canonical_python_requirements(&graph, &resolved.requirements);

    let diagnostics = validate_python_trust_policy(
        &graph,
        &app_id,
        &requirements,
        &["requests".to_string()],
        &[],
    )
    .expect_err("untrusted bridge import must fail");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED
            && diagnostic.message.contains("numpy")
    }));
}

struct ResolutionFixture {
    root: PathBuf,
}

impl ResolutionFixture {
    fn new(label: &str) -> Self {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "sifr_python_bridge_resolution_{}_{}_{}",
            std::process::id(),
            sequence,
            label
        ));
        fs::create_dir_all(&root).expect("create resolution fixture");
        Self { root }
    }

    fn package(&self, name: &str) -> crate::graph::derive::SifrPackageMetadata {
        let mut package = package(name, PythonConfig::default(), TrustPolicy::default());
        package.package_root = self.root.join(name);
        package.sifr_manifest = package.package_root.join("sifr.toml");
        fs::create_dir_all(package.package_root.join("src/python_bridges"))
            .expect("create bridge root");
        package
    }

    fn bridge_package(
        &self,
        name: &str,
        external_import: &str,
    ) -> crate::graph::derive::SifrPackageMetadata {
        let package = self.package(name);
        self.write(&package, "shared.py", "VALUE = 1\n");
        self.write(
            &package,
            "adapter.py",
            &format!("import bridge.shared\nimport {external_import}\n"),
        );
        package
    }

    fn write(
        &self,
        package: &crate::graph::derive::SifrPackageMetadata,
        relative: &str,
        source: &str,
    ) {
        let path = package
            .package_root
            .join(Path::new("src/python_bridges"))
            .join(relative);
        fs::write(path, source).expect("write bridge source");
    }
}

impl Drop for ResolutionFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
