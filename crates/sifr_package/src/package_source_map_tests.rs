use crate::cargo::metadata::parse_metadata_json;
use crate::graph::derive::{SifrPackageId, derive_package_graph};
use crate::imports::source_map::{
    DottedModulePath, PackageImportOrigin, PackageImportResolutionResult, PackageSourceMap,
};
use crate::manifest::sifr::ImportRoot;
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::DiskSourceProvider;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn package_source_map_resolves_own_and_direct_dependency_modules() {
    let temp = TestWorkspace::new("source_map_direct");
    let app = app_package(&temp);
    let math = package(
        &temp,
        "math",
        "sifr-math",
        "1.0.0",
        "math",
        None,
        &["vector"],
    );
    let graph = graph(&temp, &[&app, &math], &[edge(&app, "math", &math)]);
    let source_map =
        PackageSourceMap::build(&graph, &mut DiskSourceProvider::new()).expect("source map builds");

    let PackageImportResolutionResult::Resolved(own) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&app),
        &DottedModulePath("app.main".to_string()),
    ) else {
        panic!("own module must resolve");
    };
    assert_eq!(own.origin, PackageImportOrigin::OwnPackage);

    let PackageImportResolutionResult::Resolved(direct) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&app),
        &DottedModulePath("math.vector".to_string()),
    ) else {
        panic!("direct dependency module must resolve");
    };
    assert_eq!(direct.resolved_module.package_id, sifr_id(&math));
}

#[test]
fn transitive_dependency_import_reports_0202() {
    let temp = TestWorkspace::new("source_map_transitive");
    let app = app_package(&temp);
    let image = package(&temp, "image", "sifr-image", "0.1.0", "image", None, &[]);
    let math = package(
        &temp,
        "math",
        "sifr-math",
        "1.0.0",
        "math",
        None,
        &["vector"],
    );
    let graph = graph(
        &temp,
        &[&app, &image, &math],
        &[edge(&app, "image", &image), edge(&image, "math", &math)],
    );
    let source_map =
        PackageSourceMap::build(&graph, &mut DiskSourceProvider::new()).expect("source map builds");

    let PackageImportResolutionResult::Unresolved(diagnostic) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&app),
        &DottedModulePath("math.vector".to_string()),
    ) else {
        panic!("transitive import must remain unresolved");
    };

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_UNDECLARED_DIRECT_IMPORT
    );
}

#[test]
fn alias_import_root_remaps_to_dependency_export_root() {
    let temp = TestWorkspace::new("source_map_alias");
    let app = package(
        &temp,
        "app",
        "sifr-app",
        "0.1.0",
        "app",
        Some(r#""aliases":{"legacy_math":{"dependency":"math1","import":"math_v1"}}"#),
        &["main"],
    );
    let math = package(
        &temp,
        "math",
        "sifr-math",
        "1.0.0",
        "math",
        None,
        &["vector"],
    );
    let graph = graph(&temp, &[&app, &math], &[edge(&app, "math1", &math)]);
    let source_map =
        PackageSourceMap::build(&graph, &mut DiskSourceProvider::new()).expect("source map builds");

    let PackageImportResolutionResult::Resolved(resolved) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&app),
        &DottedModulePath("math_v1.vector".to_string()),
    ) else {
        panic!("alias must remap to the dependency export root");
    };

    assert_eq!(
        resolved.resolved_module.module_path,
        DottedModulePath("math.vector".to_string())
    );
    assert!(matches!(
        resolved.origin,
        PackageImportOrigin::DirectDependency {
            import_root: ImportRoot(ref root),
            target_export_root: ImportRoot(ref target),
            ..
        } if root == "math_v1" && target == "math"
    ));
}

#[test]
fn dotted_dependency_export_root_resolves_by_longest_scope_prefix() {
    let temp = TestWorkspace::new("source_map_dotted_export");
    let app = app_package(&temp);
    let math = package(
        &temp,
        "math",
        "sifr-math",
        "1.0.0",
        "math.core",
        None,
        &["vector"],
    );
    let graph = graph(&temp, &[&app, &math], &[edge(&app, "math", &math)]);
    let source_map =
        PackageSourceMap::build(&graph, &mut DiskSourceProvider::new()).expect("source map builds");

    let PackageImportResolutionResult::Resolved(resolved) = source_map.resolve_import_result(
        &graph,
        &sifr_id(&app),
        &DottedModulePath("math.core.vector".to_string()),
    ) else {
        panic!("dotted export root must resolve");
    };

    assert_eq!(resolved.resolved_module.package_id, sifr_id(&math));
}

#[test]
fn private_dependency_module_reports_0203() {
    let temp = TestWorkspace::new("source_map_private");
    let app = app_package(&temp);
    let math = package(
        &temp,
        "math",
        "sifr-math",
        "1.0.0",
        "math",
        None,
        &["_internal"],
    );
    let graph = graph(&temp, &[&app, &math], &[edge(&app, "math", &math)]);
    let source_map =
        PackageSourceMap::build(&graph, &mut DiskSourceProvider::new()).expect("source map builds");

    let PackageImportResolutionResult::PrivateAccess(diagnostic) = source_map
        .resolve_import_result(
            &graph,
            &sifr_id(&app),
            &DottedModulePath("math._internal".to_string()),
        )
    else {
        panic!("private dependency module must preserve private access state");
    };

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_PRIVATE_MODULE_ACCESS
    );
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

fn package(
    temp: &TestWorkspace,
    dir: &str,
    cargo_name: &str,
    version: &str,
    export: &str,
    extra_metadata: Option<&str>,
    modules: &[&str],
) -> TestPackage {
    let root = temp.package(dir);
    write_pure_package(&root, cargo_name, version, export, modules);
    TestPackage {
        root,
        cargo_name: cargo_name.to_string(),
        version: version.to_string(),
        extra_metadata: extra_metadata.map(str::to_string),
    }
}

fn app_package(temp: &TestWorkspace) -> TestPackage {
    package(temp, "app", "sifr-app", "0.1.0", "app", None, &["main"])
}

fn write_pure_package(
    package_root: &Path,
    cargo_name: &str,
    version: &str,
    export: &str,
    modules: &[&str],
) {
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
            "[package]\nname = \"{export}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
        ),
    )
    .expect("write sifr.toml");
    write_module(package_root, "__init__");
    for module in modules {
        write_module(package_root, module);
    }
}

fn write_module(package_root: &Path, module: &str) {
    if module == "__init__" || module == "main" || module.starts_with('_') {
        write_module_under(package_root, "src", module);
        return;
    }
    let mut path = package_root.join("src");
    for part in module.split('.') {
        path.push(part);
    }
    fs::create_dir_all(&path).expect("create public namespace");
    fs::write(path.join("__init__.sifr"), "").expect("write public namespace");
}

fn write_module_under(package_root: &Path, source_root: &str, module: &str) {
    let mut path = package_root.join(source_root);
    for part in module.split('.') {
        path.push(part);
    }
    fs::create_dir_all(path.parent().expect("module has parent")).expect("create module parent");
    fs::write(path.with_extension("sifr"), "").expect("write module");
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
    let extra_metadata = package
        .extra_metadata
        .as_ref()
        .map(|metadata| format!(",{metadata}"))
        .unwrap_or_default();
    format!(
        r#"{{
            "id":"{}","name":"{}","version":"{}","source":null,
            "manifest_path":"{}/Cargo.toml","dependencies":[{dependencies}],
            "targets":[{{"name":"{}","kind":["lib"],"crate_types":["lib"],"src_path":"{}/src/lib.rs"}}],
            "features":{{}},"metadata":{{"sifr":{{"manifest":"sifr.toml"{extra_metadata}}}}}
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
