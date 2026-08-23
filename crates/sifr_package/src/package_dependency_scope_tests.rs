use crate::cargo::metadata::parse_metadata_json;
use crate::graph::derive::{SifrPackageId, derive_package_graph};
use crate::graph::type_identity::{PackageTypeIdentity, TypeIdentityMismatch};
use crate::manifest::sifr::ImportRoot;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::DiskSourceProvider;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn same_import_root_can_resolve_to_different_versions_in_different_scopes() {
    let temp = TestWorkspace::new("scoped_versions");
    let app = package(&temp, "app", "sifr-app", "0.1.0", "app", None);
    let image = package(&temp, "image", "sifr-image", "0.1.0", "image", None);
    let physics = package(&temp, "physics", "sifr-physics", "0.1.0", "physics", None);
    let math_v1 = package(&temp, "math-v1", "sifr-math", "1.0.0", "math", None);
    let math_v2 = package(&temp, "math-v2", "sifr-math", "2.0.0", "math", None);

    let metadata = parse_metadata_json(&metadata_json(
        &temp.root,
        &[&app, &image, &physics, &math_v1, &math_v2],
        &[
            edge(&app, "image", &image),
            edge(&app, "physics", &physics),
            edge(&image, "math", &math_v1),
            edge(&physics, "math", &math_v2),
        ],
    ))
    .expect("metadata parses");
    let graph =
        derive_package_graph(metadata, &mut DiskSourceProvider::new()).expect("graph derives");

    let image_scope = &graph.direct_dependency_scopes[&sifr_id(&image)];
    let physics_scope = &graph.direct_dependency_scopes[&sifr_id(&physics)];

    assert_eq!(
        image_scope.imports[&ImportRoot("math".to_string())].package_id,
        sifr_id(&math_v1)
    );
    assert_eq!(
        physics_scope.imports[&ImportRoot("math".to_string())].package_id,
        sifr_id(&math_v2)
    );
}

#[test]
fn duplicate_direct_import_root_in_one_scope_reports_0201() {
    let temp = TestWorkspace::new("ambiguous_scope");
    let app = package(&temp, "app", "sifr-app", "0.1.0", "app", None);
    let math_v1 = package(&temp, "math-v1", "sifr-math", "1.0.0", "math", None);
    let math_v2 = package(&temp, "math-v2", "sifr-math", "2.0.0", "math", None);

    let metadata = parse_metadata_json(&metadata_json(
        &temp.root,
        &[&app, &math_v1, &math_v2],
        &[edge(&app, "math1", &math_v1), edge(&app, "math2", &math_v2)],
    ))
    .expect("metadata parses");
    let diagnostics = derive_package_graph(metadata, &mut DiskSourceProvider::new())
        .expect_err("ambiguous root fails");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_AMBIGUOUS_IMPORT_ROOT
    );
}

#[test]
fn direct_dependency_aliases_allow_same_export_root_in_one_scope() {
    let temp = TestWorkspace::new("aliased_scope");
    let app = package(
        &temp,
        "app",
        "sifr-app",
        "0.1.0",
        "app",
        Some(
            r#""aliases":{
                "legacy_math":{"dependency":"math1","import":"math_v1"},
                "modern_math":{"dependency":"math2","import":"math_v2"}
            }"#,
        ),
    );
    let math_v1 = package(&temp, "math-v1", "sifr-math", "1.0.0", "math", None);
    let math_v2 = package(&temp, "math-v2", "sifr-math", "2.0.0", "math", None);

    let metadata = parse_metadata_json(&metadata_json(
        &temp.root,
        &[&app, &math_v1, &math_v2],
        &[edge(&app, "math1", &math_v1), edge(&app, "math2", &math_v2)],
    ))
    .expect("metadata parses");
    let graph = derive_package_graph(metadata, &mut DiskSourceProvider::new())
        .expect("aliases disambiguate");
    let app_scope = &graph.direct_dependency_scopes[&sifr_id(&app)];

    assert!(
        !app_scope
            .imports
            .contains_key(&ImportRoot("math".to_string()))
    );
    assert_eq!(
        app_scope.imports[&ImportRoot("math_v1".to_string())].package_id,
        sifr_id(&math_v1)
    );
    assert_eq!(
        app_scope.imports[&ImportRoot("math_v2".to_string())].package_id,
        sifr_id(&math_v2)
    );
}

#[test]
fn type_identity_mismatch_reports_0204_for_distinct_package_instances() {
    let expected = PackageTypeIdentity {
        package_id: SifrPackageId("sifr-math@1.0.0#registry".to_string()),
        cargo_package_id: crate::CargoPackageId("registry+sifr-math@1.0.0".to_string()),
        module_path: "math.vector".to_string(),
        type_name: "Vector".to_string(),
        dependency_path: vec![crate::CargoPackageId(
            "registry+sifr-math@1.0.0".to_string(),
        )],
    };
    let actual = PackageTypeIdentity {
        package_id: SifrPackageId("sifr-math@2.0.0#registry".to_string()),
        cargo_package_id: crate::CargoPackageId("registry+sifr-math@2.0.0".to_string()),
        module_path: "math.vector".to_string(),
        type_name: "Vector".to_string(),
        dependency_path: vec![crate::CargoPackageId(
            "registry+sifr-math@2.0.0".to_string(),
        )],
    };

    let diagnostic = TypeIdentityMismatch { expected, actual }.diagnostic();

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_TYPE_IDENTITY_MISMATCH
    );
    assert!(diagnostic.message.contains("sifr-math@1.0.0#registry"));
    assert!(diagnostic.message.contains("sifr-math@2.0.0#registry"));
    assert!(diagnostic.message.contains("registry+sifr-math@1.0.0"));
    assert!(diagnostic.message.contains("registry+sifr-math@2.0.0"));
}

fn package(
    temp: &TestWorkspace,
    dir: &str,
    cargo_name: &str,
    version: &str,
    sifr_name: &str,
    extra_metadata: Option<&str>,
) -> TestPackage {
    let root = temp.package(dir);
    write_pure_package(&root, cargo_name, version, sifr_name);
    TestPackage {
        root,
        cargo_name: cargo_name.to_string(),
        version: version.to_string(),
        extra_metadata: extra_metadata.map(str::to_string),
    }
}

fn write_pure_package(package_root: &Path, cargo_name: &str, version: &str, sifr_name: &str) {
    fs::create_dir_all(package_root.join("src")).expect("create src");
    fs::write(
        package_root.join("src/lib.rs"),
        "// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.\n",
    )
    .expect("write marker");
    fs::write(
        package_root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{cargo_name}\"\nversion = \"{version}\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
        ),
    )
    .expect("write Cargo.toml");
    fs::write(
        package_root.join("sifr.toml"),
        format!(
            "[package]\nname = \"{sifr_name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
        ),
    )
    .expect("write sifr.toml");
    fs::write(package_root.join("src/__init__.sifr"), "").expect("write init");
}

fn metadata_json(
    workspace_root: &Path,
    packages: &[&TestPackage],
    edges: &[ResolveEdge],
) -> String {
    let package_json = packages
        .iter()
        .map(|package| metadata_package_json(package, edges))
        .collect::<Vec<_>>()
        .join(",");
    let members = packages
        .iter()
        .map(|package| format!(r#""{}""#, cargo_id(package)))
        .collect::<Vec<_>>()
        .join(",");
    let nodes = packages
        .iter()
        .map(|package| {
            let deps = edges
                .iter()
                .filter(|edge| edge.from == cargo_id(package))
                .map(|edge| {
                    format!(
                        r#"{{"name":"{}","pkg":"{}","dep_kinds":[]}}"#,
                        edge.name, edge.to
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"id":"{}","dependencies":[{}],"deps":[{}],"features":[]}}"#,
                cargo_id(package),
                edges
                    .iter()
                    .filter(|edge| edge.from == cargo_id(package))
                    .map(|edge| format!(r#""{}""#, edge.to))
                    .collect::<Vec<_>>()
                    .join(","),
                deps
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{
            "packages":[{package_json}],
            "resolve":{{"nodes":[{nodes}]}},
            "workspace_members":[{members}],
            "target_directory":"{}/target",
            "workspace_root":"{}"
        }}"#,
        workspace_root.display(),
        workspace_root.display()
    )
}

fn metadata_package_json(package: &TestPackage, edges: &[ResolveEdge]) -> String {
    let dependency_json = edges
        .iter()
        .filter(|edge| edge.from == cargo_id(package))
        .map(|edge| {
            format!(
                r#"{{
                    "name":"{}",
                    "package":"{}",
                    "req":"*",
                    "kind":null,
                    "target":null,
                    "uses_workspace":false
                }}"#,
                edge.name, edge.package
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let extra_metadata = package
        .extra_metadata
        .as_ref()
        .map(|metadata| format!(",{metadata}"))
        .unwrap_or_default();
    format!(
        r#"{{
            "id":"{}",
            "name":"{}",
            "version":"{}",
            "source":null,
            "manifest_path":"{}/Cargo.toml",
            "dependencies":[{dependency_json}],
            "targets":[{{
                "name":"{}",
                "kind":["lib"],
                "crate_types":["lib"],
                "src_path":"{}/src/lib.rs"
            }}],
            "features":{{}},
            "metadata":{{"sifr":{{"manifest":"sifr.toml"{extra_metadata}}}}}
        }}"#,
        cargo_id(package),
        package.cargo_name,
        package.version,
        package.root.display(),
        package.cargo_name,
        package.root.display()
    )
}

fn edge(from: &TestPackage, name: &str, to: &TestPackage) -> ResolveEdge {
    ResolveEdge {
        from: cargo_id(from),
        name: name.to_string(),
        package: to.cargo_name.clone(),
        to: cargo_id(to),
    }
}

fn cargo_id(package: &TestPackage) -> String {
    format!(
        "path+file://{}#{}@{}",
        package.root.display(),
        package.cargo_name,
        package.version
    )
}

fn sifr_id(package: &TestPackage) -> SifrPackageId {
    SifrPackageId(format!("{}@{}#path", package.cargo_name, package.version))
}

struct ResolveEdge {
    from: String,
    name: String,
    package: String,
    to: String,
}

struct TestPackage {
    root: PathBuf,
    cargo_name: String,
    version: String,
    extra_metadata: Option<String>,
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("sifr_package_{name}_{nonce}"));
        fs::create_dir_all(&root).expect("create temp workspace");
        Self { root }
    }

    fn package(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir_all(&path).expect("create package");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
