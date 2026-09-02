use crate::test_support::TestUnwrap as _;

use crate::cargo::metadata::parse_metadata_json;
use crate::graph::derive::{SifrPackageId, derive_package_graph};
use crate::imports::source_map::{
    DottedModulePath, PackageImportOrigin, PackageImportResolutionResult, PackageSourceMap,
};
use sifr_frontend::DiskSourceProvider;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn import_resolution_result_preserves_ambiguous_candidates() {
    let temp = TestWorkspace::new("source_map_ambiguous_result");
    let app = package(&temp, "app", "sifr-app", "0.1.0", "app", &["main"]);
    let math = package(&temp, "math", "sifr-math", "1.0.0", "math", &["vector"]);
    write_module_under(&math.root, "src/vector", "__init__");

    let graph = graph(&temp, &[&app, &math], &[edge(&app, "math", &math)]);
    let source_map = PackageSourceMap::build(&graph, &mut DiskSourceProvider::new())
        .test_unwrap("ambiguous map is queryable");

    let PackageImportResolutionResult::Ambiguous(ambiguity) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&app),
        &DottedModulePath("math.vector".to_string()),
    ) else {
        panic!("expected ambiguous package import result");
    };

    assert_eq!(ambiguity.candidates.len(), 2);
    assert!(matches!(
        ambiguity.origin,
        PackageImportOrigin::DirectDependency { .. }
    ));
}

#[test]
fn import_resolution_result_distinguishes_unresolved_private_and_fatal_states() {
    let temp = TestWorkspace::new("source_map_resolution_states");
    let app = package(&temp, "app", "sifr-app", "0.1.0", "app", &["main"]);
    let math = package(&temp, "math", "sifr-math", "1.0.0", "math", &["_internal"]);
    let graph = graph(&temp, &[&app, &math], &[edge(&app, "math", &math)]);
    let source_map = PackageSourceMap::build(&graph, &mut DiskSourceProvider::new())
        .test_unwrap("source map builds");

    assert!(matches!(
        source_map.resolve_import_result(
            &graph,
            &sifr_id(&app),
            &DottedModulePath("missing.module".to_string())
        ),
        PackageImportResolutionResult::Unresolved(_)
    ));
    assert!(matches!(
        source_map.resolve_import_result(
            &graph,
            &sifr_id(&app),
            &DottedModulePath("math._internal".to_string())
        ),
        PackageImportResolutionResult::PrivateAccess(_)
    ));

    let fatal = PackageSourceMap::from_fatal_diagnostics(vec![
        crate::PackageDiagnostic::cargo_metadata_parse("synthetic fatal package map failure"),
    ]);
    assert!(matches!(
        fatal.resolve_import_result(
            &graph,
            &sifr_id(&app),
            &DottedModulePath("math.vector".to_string())
        ),
        PackageImportResolutionResult::FatalPackageMapFailure(_)
    ));
}

fn graph(
    temp: &TestWorkspace,
    packages: &[&TestPackage],
    edges: &[ResolveEdge],
) -> crate::SifrPackageGraph {
    let metadata = parse_metadata_json(&metadata_json(&temp.root, packages, edges))
        .test_unwrap("metadata parses");
    derive_package_graph(metadata, &mut DiskSourceProvider::new()).test_unwrap("graph derives")
}

fn package(
    temp: &TestWorkspace,
    dir: &str,
    cargo_name: &str,
    version: &str,
    export: &str,
    modules: &[&str],
) -> TestPackage {
    let root = temp.package(dir);
    fs::create_dir_all(root.join("src")).test_unwrap("create src");
    fs::write(
        root.join("src/lib.rs"),
        "// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.\n",
    )
    .test_unwrap("write marker");
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{cargo_name}\"\nversion = \"{version}\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
        ),
    )
    .test_unwrap("write Cargo.toml");
    fs::write(
        root.join("sifr.toml"),
        format!(
            "[package]\nname = \"{export}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
        ),
    )
    .test_unwrap("write sifr.toml");
    write_module_under(&root, "src", "__init__");
    for module in modules {
        write_module_under(&root, "src", module);
    }
    TestPackage {
        root,
        cargo_name: cargo_name.to_string(),
        version: version.to_string(),
    }
}

fn write_module_under(package_root: &Path, source_root: &str, module: &str) {
    let mut path = package_root.join(source_root);
    for part in module.split('.') {
        path.push(part);
    }
    fs::create_dir_all(path.parent().test_unwrap("module has parent"))
        .test_unwrap("create module parent");
    fs::write(path.with_extension("sifr"), "").test_unwrap("write module");
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
        to: cargo_id(to),
        package: to.cargo_name.clone(),
    }
}

fn sifr_id(package: &TestPackage) -> SifrPackageId {
    SifrPackageId(format!("{}@{}#path", package.cargo_name, package.version))
}

fn cargo_id(package: &TestPackage) -> String {
    format!(
        "path+file://{}#{}@{}",
        package.root.display(),
        package.cargo_name,
        package.version
    )
}

struct ResolveEdge {
    from: String,
    name: String,
    to: String,
    package: String,
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
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .test_unwrap("time should move forward")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("sifr_pkg_import_resolution_{name}_{stamp}"));
        fs::create_dir_all(&root).test_unwrap("create temp workspace");
        Self { root }
    }

    fn package(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir_all(&path).test_unwrap("create temp package");
        path
    }
}
