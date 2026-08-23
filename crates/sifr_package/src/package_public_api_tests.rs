use crate::CargoPackageId;
use crate::cargo::metadata::parse_metadata_json;
use crate::graph::derive::{SifrPackageId, derive_package_graph};
use crate::imports::source_map::{
    DottedModulePath, PackageImportOrigin, PackageImportResolutionResult, PackageSourceMap,
};
use crate::manifest::sifr::SifrManifest;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::DiskSourceProvider;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn source_layout_public_namespaces_derive_from_init_sifr() {
    let temp = TestWorkspace::new("source_layout_public_namespaces");
    let app = production_package(&temp, "app", "sifr-app", "0.1.0", "demo_app");
    let json = production_package(&temp, "json", "sifr-demo-json", "1.0.0", "demo_json");
    write_src_file(
        &json.root,
        "__init__.sifr",
        "from .parse import parse_json\nfrom . import codecs\n",
    );
    write_src_file(
        &json.root,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    write_src_file(
        &json.root,
        "codecs/__init__.sifr",
        "from .json import decode_json\n",
    );
    write_src_file(
        &json.root,
        "codecs/json.sifr",
        "def decode_json() -> int:\n    return 1\n",
    );
    let graph = graph(&temp, &[&app, &json], &[edge(&app, "demo_json", &json)]);
    let source_map =
        PackageSourceMap::build(&graph, &mut DiskSourceProvider::new()).expect("source map builds");

    let PackageImportResolutionResult::Resolved(root) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&app),
        &DottedModulePath("demo_json".to_string()),
    ) else {
        panic!("public package root must resolve");
    };
    assert_eq!(
        root.origin,
        PackageImportOrigin::DirectDependency {
            import_root: crate::ImportRoot("demo_json".to_string()),
            target_export_root: crate::ImportRoot("demo_json".to_string()),
            dependency_package_id: sifr_id(&json),
        }
    );

    assert!(matches!(
        source_map.resolve_import_result(
            &graph,
            &sifr_id(&app),
            &DottedModulePath("demo_json.codecs".to_string()),
        ),
        PackageImportResolutionResult::Resolved(_)
    ));

    let PackageImportResolutionResult::PrivateAccess(diagnostic) = source_map
        .resolve_import_result(
            &graph,
            &sifr_id(&app),
            &DottedModulePath("demo_json.parse".to_string()),
        )
    else {
        panic!("implementation module must preserve private access state");
    };
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_PRIVATE_MODULE_ACCESS
    );
}

#[test]
fn local_package_can_import_own_private_implementation_module() {
    let temp = TestWorkspace::new("source_layout_local_private");
    let json = production_package(&temp, "json", "sifr-demo-json", "1.0.0", "demo_json");
    write_src_file(
        &json.root,
        "__init__.sifr",
        "from .parse import parse_json\n",
    );
    write_src_file(
        &json.root,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    let graph = graph(&temp, &[&json], &[]);
    let source_map =
        PackageSourceMap::build(&graph, &mut DiskSourceProvider::new()).expect("source map builds");

    let PackageImportResolutionResult::Resolved(own) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&json),
        &DottedModulePath("demo_json.parse".to_string()),
    ) else {
        panic!("own implementation module must resolve");
    };
    assert_eq!(own.origin, PackageImportOrigin::OwnPackage);
}

#[test]
fn duplicate_init_public_symbol_reports_0713() {
    let temp = TestWorkspace::new("source_layout_duplicate_public_symbol");
    let json = production_package(&temp, "json", "sifr-demo-json", "1.0.0", "demo_json");
    write_src_file(
        &json.root,
        "__init__.sifr",
        "from .parse import parse_json\nfrom .value import parse_json\n",
    );
    write_src_file(&json.root, "parse.sifr", "");
    write_src_file(&json.root, "value.sifr", "");
    let graph = graph(&temp, &[&json], &[]);
    let diagnostics = PackageSourceMap::build(&graph, &mut DiskSourceProvider::new())
        .expect_err("duplicate public symbol fails");

    assert!(
        diagnostics.iter().any(
            |diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_DUPLICATE_PUBLIC_API_SYMBOL
        )
    );
}

#[test]
fn production_manifest_defaults_source_root_to_src_and_preserves_unknown_keys() {
    let temp = TestWorkspace::new("production_manifest_defaults");
    let manifest_path = temp.root.join("sifr.toml");
    let manifest = SifrManifest::parse(
        &CargoPackageId("path+file:///tmp/demo#sifr-demo@0.1.0".to_string()),
        &manifest_path,
        "[package]\nname = \"demo\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[tooling]\nunknown = true\n",
    )
    .expect("manifest parses");

    assert_eq!(manifest.source_root.0, PathBuf::from("src"));
}

#[test]
fn manifest_exports_use_normal_unsupported_field_diagnostic() {
    let diagnostic = SifrManifest::parse(
        &CargoPackageId("path+file:///tmp/demo#sifr-demo@0.1.0".to_string()),
        Path::new("/tmp/demo/sifr.toml"),
        "[package]\nname = \"demo\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[exports]\nmodules = [\"demo\"]\n",
    )
    .expect_err("production exports are rejected");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST
    );
    assert!(
        diagnostic
            .message
            .contains("invalid sifr.toml key 'exports'")
    );
}

#[test]
fn manifest_bin_tables_use_normal_unsupported_field_diagnostic() {
    let diagnostic = SifrManifest::parse(
        &CargoPackageId("path+file:///tmp/demo#sifr-demo@0.1.0".to_string()),
        Path::new("/tmp/demo/sifr.toml"),
        "[package]\nname = \"demo\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[[bin]]\nname = \"demo\"\npath = \"src/main.sifr\"\n",
    )
    .expect_err("production bin tables are rejected");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST
    );
    assert!(diagnostic.message.contains("invalid sifr.toml key 'bin'"));
}

fn graph(
    temp: &TestWorkspace,
    packages: &[&TestPackage],
    edges: &[ResolveEdge],
) -> crate::SifrPackageGraph {
    let metadata =
        parse_metadata_json(&metadata_json(&temp.root, packages, edges)).expect("metadata parses");
    derive_package_graph(metadata, &mut DiskSourceProvider::new()).expect("graph derives")
}

fn production_package(
    temp: &TestWorkspace,
    dir: &str,
    cargo_name: &str,
    version: &str,
    sifr_name: &str,
) -> TestPackage {
    let root = temp.package(dir);
    fs::create_dir_all(root.join("src")).expect("create src");
    fs::write(
        root.join("src/lib.rs"),
        "// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.\n",
    )
    .expect("write marker");
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{cargo_name}\"\nversion = \"{version}\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
        ),
    )
    .expect("write Cargo.toml");
    fs::write(
        root.join("sifr.toml"),
        format!(
            "[package]\nname = \"{sifr_name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
        ),
    )
    .expect("write sifr.toml");
    write_src_file(&root, "__init__.sifr", "");
    TestPackage {
        root,
        cargo_name: cargo_name.to_string(),
        version: version.to_string(),
    }
}

fn write_src_file(package_root: &Path, relative: &str, source: &str) {
    let path = package_root.join("src").join(relative);
    fs::create_dir_all(path.parent().expect("source has parent")).expect("create source parent");
    fs::write(path, source).expect("write source");
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
                .map(|edge| format!(r#"{{"name":"{}","pkg":"{}"}}"#, edge.name, edge.to))
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"id":"{}","deps":[{deps}]}}"#, cargo_id(package))
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"packages":[{package_json}],"resolve":{{"nodes":[{nodes}]}},"workspace_members":[{members}],"target_directory":"{}/target","workspace_root":"{}"}}"#,
        workspace_root.display(),
        workspace_root.display()
    )
}

fn metadata_package_json(package: &TestPackage, edges: &[ResolveEdge]) -> String {
    let dependencies = edges
        .iter()
        .filter(|edge| edge.from == cargo_id(package))
        .map(|edge| {
            format!(
                r#"{{"name":"{}","package":"{}","req":"*","kind":null,"target":null,"uses_workspace":false}}"#,
                edge.name, edge.package
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{
            "id":"{}","name":"{}","version":"{}","source":null,
            "manifest_path":"{}/Cargo.toml","dependencies":[{dependencies}],
            "targets":[{{"name":"{}","kind":["lib"],"crate_types":["lib"],"src_path":"{}/src/lib.rs"}}],
            "features":{{}},"metadata":{{"sifr":{{"manifest":"sifr.toml"}}}}
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
