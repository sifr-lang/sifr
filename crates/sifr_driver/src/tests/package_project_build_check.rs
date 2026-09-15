use super::project_build_check::mktemp_dir;
use crate::{PackageEntrypoint, build_cached_package_project, check_package_project};
use sifr_diagnostics::DiagnosticCode;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

#[path = "required_error_message_python.rs"]
mod required_error_message_python;

#[derive(Clone)]
struct TestPackage {
    root: PathBuf,
    cargo_name: String,
    version: String,
    sifr_name: String,
    aliases: Vec<TestAlias>,
    has_sifr_manifest: bool,
}

#[derive(Clone)]
struct TestEdge {
    from: String,
    dependency_name: String,
    to_package_id: String,
}

#[derive(Clone)]
struct TestAlias {
    alias: String,
    dependency: String,
    import: String,
}

fn production_package(
    workspace: &Path,
    dir_name: &str,
    cargo_name: &str,
    sifr_name: &str,
) -> TestPackage {
    production_package_version(workspace, dir_name, cargo_name, "0.1.0", sifr_name)
}

fn production_package_version(
    workspace: &Path,
    dir_name: &str,
    cargo_name: &str,
    version: &str,
    sifr_name: &str,
) -> TestPackage {
    let root = workspace.join(dir_name);
    std::fs::create_dir_all(root.join("src")).expect("package src dir should be created");
    std::fs::write(root.join("src/lib.rs"), "").expect("pure marker should be written");
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"{cargo_name}\"\nversion = \"{version}\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
        ),
    )
    .expect("cargo manifest should be written");
    std::fs::write(
        root.join("sifr.toml"),
        format!(
            "[package]\nname = \"{sifr_name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
        ),
    )
    .expect("sifr manifest should be written");
    TestPackage {
        root,
        cargo_name: cargo_name.to_string(),
        version: version.to_string(),
        sifr_name: sifr_name.to_string(),
        aliases: Vec::new(),
        has_sifr_manifest: true,
    }
}

fn backend_rust_package(workspace: &Path, dir_name: &str, cargo_name: &str) -> TestPackage {
    let root = workspace.join(dir_name);
    std::fs::create_dir_all(root.join("src")).expect("backend src dir should be created");
    std::fs::write(root.join("src/lib.rs"), "").expect("backend marker should be written");
    std::fs::write(
        root.join("Cargo.toml"),
        format!("[package]\nname = \"{cargo_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n"),
    )
    .expect("backend cargo manifest should be written");
    TestPackage {
        root,
        cargo_name: cargo_name.to_string(),
        version: "0.1.0".to_string(),
        sifr_name: String::new(),
        aliases: Vec::new(),
        has_sifr_manifest: false,
    }
}

fn write_manifest_dependency_alias(package: &TestPackage, dependency_name: &str, import: &str) {
    let manifest = package.root.join("sifr.toml");
    let mut source = std::fs::read_to_string(&manifest).expect("manifest should be readable");
    writeln!(
        source,
        "\n[dependencies]\n{dependency_name} = {{ package = \"{dependency_name}\", path = \"../{dependency_name}\", import = \"{import}\" }}"
    )
    .expect("writing to a String should succeed");
    std::fs::write(manifest, source).expect("manifest should be updated");
}

fn write_package_source(package: &TestPackage, relative: &str, source: &str) {
    let path = package.root.join("src").join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("source parent should be created");
    }
    std::fs::write(path, source).expect("package source should be written");
}

fn package_edge(from: &TestPackage, dependency_name: &str, to: &TestPackage) -> TestEdge {
    TestEdge {
        from: from.cargo_name.clone(),
        dependency_name: dependency_name.to_string(),
        to_package_id: cargo_package_id(to),
    }
}

fn cargo_package_id(package: &TestPackage) -> String {
    format!(
        "path+file://{}#{}@{}",
        package.root.display(),
        package.cargo_name,
        package.version
    )
}

fn package_graph(
    workspace: &Path,
    packages: &[&TestPackage],
    edges: &[TestEdge],
) -> sifr_package::SifrPackageGraph {
    let package_json = packages
        .iter()
        .map(|package| {
            let dependencies = edges
                .iter()
                .filter(|edge| edge.from == package.cargo_name)
                .map(|edge| {
                    let target = packages
                        .iter()
                        .find(|candidate| cargo_package_id(candidate) == edge.to_package_id)
                        .expect("edge target should exist");
                    serde_json::json!({
                        "name": edge.dependency_name,
                        "package": target.cargo_name,
                        "req": "*",
                        "kind": null,
                        "target": null,
                        "uses_workspace": false
                    })
                })
                .collect::<Vec<_>>();
            let aliases = package
                .aliases
                .iter()
                .map(|alias| {
                    (
                        alias.alias.clone(),
                        serde_json::json!({
                            "dependency": alias.dependency,
                            "import": alias.import
                        }),
                    )
                })
                .collect::<serde_json::Map<_, _>>();
            let metadata = if package.has_sifr_manifest {
                serde_json::json!({ "sifr": { "manifest": "sifr.toml", "aliases": aliases } })
            } else {
                serde_json::json!({})
            };
            serde_json::json!({
                "id": cargo_package_id(package),
                "name": package.cargo_name,
                "version": package.version,
                "source": null,
                "manifest_path": package.root.join("Cargo.toml").display().to_string(),
                "dependencies": dependencies,
                "targets": [{
                    "name": package.cargo_name,
                    "kind": ["lib"],
                    "crate_types": ["lib"],
                    "src_path": package.root.join("src/lib.rs").display().to_string()
                }],
                "features": {},
                "metadata": metadata
            })
        })
        .collect::<Vec<_>>();
    let resolve_nodes = packages
        .iter()
        .map(|package| {
            let deps = edges
                .iter()
                .filter(|edge| edge.from == package.cargo_name)
                .map(|edge| {
                    let target = packages
                        .iter()
                        .find(|candidate| cargo_package_id(candidate) == edge.to_package_id)
                        .expect("edge target should exist");
                    serde_json::json!({
                        "name": edge.dependency_name,
                        "pkg": cargo_package_id(target)
                    })
                })
                .collect::<Vec<_>>();
            serde_json::json!({
                "id": cargo_package_id(package),
                "deps": deps
            })
        })
        .collect::<Vec<_>>();
    let metadata = serde_json::json!({
        "packages": package_json,
        "resolve": { "nodes": resolve_nodes },
        "workspace_members": packages.iter().map(|package| cargo_package_id(package)).collect::<Vec<_>>(),
        "target_directory": workspace.join("target").display().to_string(),
        "workspace_root": workspace.display().to_string()
    });
    let metadata = sifr_package::parse_metadata_json(&metadata.to_string())
        .expect("metadata json should parse");
    sifr_package::derive_package_graph(metadata, &mut sifr_frontend::DiskSourceProvider::new())
        .expect("package graph should derive")
}

fn package_entrypoint(
    graph: &sifr_package::SifrPackageGraph,
    source_map: &sifr_package::PackageSourceMap,
    package: &TestPackage,
    main_file: PathBuf,
) -> PackageEntrypoint {
    let package_id = graph
        .packages
        .values()
        .find(|metadata| metadata.sifr_name.0 == package.sifr_name)
        .expect("package should be in graph")
        .package_id
        .clone();
    PackageEntrypoint {
        main_file,
        package_id,
        graph: graph.clone(),
        source_map: source_map.clone(),
        python_runtime: None,
        lock_mode: sifr_package::CargoLockMode::Normal,
    }
}

#[path = "package_project_build_check_support.rs"]
mod support;
use support::{local_python_runtime, local_python_runtime_with_roots};
#[path = "package_python_async_runtime_tests.rs"]
mod python_async_runtime_tests;
#[path = "package_python_bridge_archive_tests.rs"]
mod python_bridge_archive_tests;
#[path = "package_python_raw_api_tests.rs"]
mod python_raw_api_tests;
#[path = "package_rust_interop_build_tests.rs"]
mod rust_interop_build_tests;

#[test]
fn test_check_package_project_resolves_public_namespace_reexports() {
    let dir = mktemp_dir("package_public_reexports");
    let mut app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    app.aliases.push(TestAlias {
        alias: "demo_json_v1".to_string(),
        dependency: "demo_json_v1".to_string(),
        import: "demo_json_v1".to_string(),
    });
    let json = production_package(&dir, "json", "sifr-demo-json", "demo_json");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_json_v1 import decode_json, parse_json\n\n\
def main():\n    assert parse_json() == 1\n    assert decode_json() == 2\n",
    );
    write_package_source(
        &json,
        "__init__.sifr",
        "from .parse import parse_json\nfrom .codecs import decode_json\n",
    );
    write_package_source(
        &json,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    write_package_source(
        &json,
        "codecs/__init__.sifr",
        "from .json import decode_json\n",
    );
    write_package_source(
        &json,
        "codecs/json.sifr",
        "def decode_json() -> int:\n    return 2\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &json],
        &[package_edge(&app, "demo_json_v1", &json)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(
        errors.is_empty(),
        "package public namespace imports should succeed: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn package_source_cannot_declare_compiler_intrinsics() {
    let dir = mktemp_dir("package_compiler_intrinsic_rejected");
    let mut app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    app.aliases.push(TestAlias {
        alias: "demo_library".to_string(),
        dependency: "sifr-demo-library".to_string(),
        import: "demo_library".to_string(),
    });
    let library = production_package(
        &dir,
        "sifr-demo-library",
        "sifr-demo-library",
        "demo_library",
    );
    write_manifest_dependency_alias(&app, "sifr-demo-library", "demo_library");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_library import forbidden\n\ndef main() -> None:\n    forbidden(True)\n",
    );
    write_package_source(
        &library,
        "__init__.sifr",
        "@compiler_intrinsic(test_assert_true)\ndef forbidden(value: bool) -> None:\n    ...\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &library],
        &[package_edge(&app, "sifr-demo-library", &library)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(errors.iter().any(|error| {
        error.code == DiagnosticCode::TYPE_UNSUPPORTED_EXPRESSION_FORM.code()
            && error
                .message
                .contains("reserved for canonical public sysroot declarations")
    }));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn package_python_bridge_target_activates_only_for_owning_package() {
    let dir = mktemp_dir("package_python_bridge_target");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    write_package_source(
        &app,
        "main.sifr",
        "from sifr.python import PythonError\n\n@python(bridge.adapter.value)\ndef value() -> Result[int, PythonError]: ...\n\ndef main():\n    pass\n",
    );
    write_package_source(
        &app,
        "python_bridges/adapter.py",
        "def value():\n    return 42\n",
    );
    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(
        errors.is_empty(),
        "package-owned bridge target should activate: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_cached_package_project_materializes_namespace_roots() {
    let dir = mktemp_dir("package_namespace_build");
    let mut app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    app.aliases.push(TestAlias {
        alias: "demo_json_v1".to_string(),
        dependency: "demo_json_v1".to_string(),
        import: "demo_json_v1".to_string(),
    });
    let json = production_package(&dir, "json", "sifr-demo-json", "demo_json");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_json_v1 import parse_json\n\n\
def main():\n    print(parse_json())\n",
    );
    write_package_source(&json, "__init__.sifr", "from .parse import parse_json\n");
    write_package_source(
        &json,
        "parse.sifr",
        "def parse_json() -> int:\n    return 7\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &json],
        &[package_edge(&app, "demo_json_v1", &json)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let artifact =
        build_cached_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("package namespace root project should build");
    let generated_project_root = generated_project_root(artifact.binary_path());
    assert!(
        !generated_project_root.join(".cargo/config.toml").exists(),
        "package-owned generated builds should not copy sysroot Cargo config"
    );
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("package binary should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "7");
    let _ = std::fs::remove_dir_all(dir);
}

fn generated_project_root(binary_path: &Path) -> PathBuf {
    binary_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("cached binary path should be <project>/target/release/<bin>")
        .to_path_buf()
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_const_specialization_without_structural_runtime() {
    let dir = mktemp_dir("package_const_specialization_without_structural_runtime");
    let app = production_package(&dir, "app", "sifr-const-app", "const_app");
    let meta = production_package(&dir, "meta", "sifr-const-meta", "const_meta");
    write_package_source(
        &meta,
        "__init__.sifr",
        "class IssueArgs:\n    problem: str\n\nclass Issue:\n    package: str\n    reason_code: str\n    severity: str\n    arguments: IssueArgs\n    notes: list[str]\n\nclass Outcome:\n    status: str\n    value: str | None\n    issues: list[Issue]\n\n@const_eval\ndef describe(shape: dict[str, str]) -> Outcome:\n    return Outcome(\"produced\", shape[\"canonical_identity\"], [])\n",
    );
    write_package_source(
        &app,
        "main.sifr",
        "from const_meta import describe\n\n@const_specialize(\"const_meta\", \"describe\")\nclass Counter:\n    count: int\n\ndef main() -> None:\n    value: Counter = Counter(7)\n    assert value.count == 7\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &meta],
        &[package_edge(&app, "const_meta", &meta)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    let artifact =
        build_cached_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("non-structural const specialization should build");
    let generated =
        std::fs::read_to_string(generated_project_root(artifact.binary_path()).join("src/main.rs"))
            .expect("generated source should be retained");
    assert!(!generated.contains("sifr_runtime::interop::structural"));
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("non-structural specialization binary should run");
    assert!(output.status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_cached_package_project_links_direct_rust_interop_dependency() {
    let dir = mktemp_dir("package_direct_rust_interop");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let crc32fast = backend_rust_package(&dir, "crc32fast", "crc32fast");
    std::fs::write(
        crc32fast.root.join("src/lib.rs"),
        "pub fn hash(data: &[u8]) -> u32 {\n    data.iter().fold(0_u32, |acc, byte| acc.wrapping_add(u32::from(*byte)))\n}\n",
    )
    .expect("backend source should be written");
    std::fs::write(
        app.root.join("sifr.toml"),
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[rust]\ndirect-crate-bindings = true\n\n[trust]\nrust-no-panic = [\"crc32fast.hash\"]\n",
    )
    .expect("app manifest should enable direct rust interop");
    write_package_source(
        &app,
        "main.sifr",
        "@rust(crc32fast.hash, panic=trusted_no_panic)\n\
def crc32(data: bytes) -> uint32:\n    zero: uint32 = 0\n    return zero\n\n\
def main():\n    print(crc32(b\"abc\"))\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &crc32fast],
        &[package_edge(&app, "crc32fast", &crc32fast)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let artifact =
        build_cached_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("direct Rust interop project should build");
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("package binary should run");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "294");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_package_keeps_runtime_and_source_python_error_identities_distinct() {
    let dir = mktemp_dir("package_python_error_identity_collision");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    write_package_source(
        &app,
        "main.sifr",
        "from sifr.python import PythonError as RuntimePythonError\n\n\
class PythonError(Error):\n    message: str\n    code: int\n\n\
def describe(error: RuntimePythonError) -> str:\n    return error.kind\n\n\
def main() -> None:\n    local: PythonError = PythonError(\"local\", 7)\n    assert local.code == 7\n",
    );
    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let mut entrypoint =
        package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));
    entrypoint.python_runtime = Some(local_python_runtime(&dir));

    let artifact =
        build_cached_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("canonical and source PythonError declarations should build together");
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("package binary should run");

    assert!(
        output.status.success(),
        "binary should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_uses_sifr_manifest_dependency_aliases() {
    let dir = mktemp_dir("package_sifr_manifest_alias");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    write_manifest_dependency_alias(&app, "demo_json_v1", "demo_json_v1");
    let json = production_package(&dir, "json", "sifr-demo-json", "demo_json");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_json_v1 import parse_json\n\n\
def main():\n    assert parse_json() == 1\n",
    );
    write_package_source(&json, "__init__.sifr", "from .parse import parse_json\n");
    write_package_source(
        &json,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &json],
        &[package_edge(&app, "demo_json_v1", &json)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(
        errors.is_empty(),
        "Sifr manifest dependency aliases should resolve: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_namespaces_transitive_package_versions() {
    let dir = mktemp_dir("package_transitive_version_namespace");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let image = production_package(&dir, "image", "sifr-demo-image", "demo_image");
    let physics = production_package(&dir, "physics", "sifr-demo-physics", "demo_physics");
    let math_v1 =
        production_package_version(&dir, "math-v1", "sifr-demo-math", "1.0.0", "demo_math");
    let math_v2 =
        production_package_version(&dir, "math-v2", "sifr-demo-math", "2.0.0", "demo_math");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_image import image_value\nfrom demo_physics import physics_value\n\n\
def main():\n    assert image_value() == 1\n    assert physics_value() == 2\n",
    );
    write_package_source(&image, "__init__.sifr", "from .api import image_value\n");
    write_package_source(
        &image,
        "api.sifr",
        "from demo_math import math_value\n\n\
def image_value() -> int:\n    return math_value()\n",
    );
    write_package_source(
        &physics,
        "__init__.sifr",
        "from .api import physics_value\n",
    );
    write_package_source(
        &physics,
        "api.sifr",
        "from demo_math import math_value\n\n\
def physics_value() -> int:\n    return math_value()\n",
    );
    write_package_source(&math_v1, "__init__.sifr", "from .value import math_value\n");
    write_package_source(
        &math_v1,
        "value.sifr",
        "def math_value() -> int:\n    return 1\n",
    );
    write_package_source(&math_v2, "__init__.sifr", "from .value import math_value\n");
    write_package_source(
        &math_v2,
        "value.sifr",
        "def math_value() -> int:\n    return 2\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &image, &physics, &math_v1, &math_v2],
        &[
            package_edge(&app, "demo_image", &image),
            package_edge(&app, "demo_physics", &physics),
            package_edge(&image, "demo_math", &math_v1),
            package_edge(&physics, "demo_math", &math_v2),
        ],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(
        errors.is_empty(),
        "transitive package versions should compile in distinct namespaces: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_rejects_private_dependency_module() {
    let dir = mktemp_dir("package_private_rejection");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let json = production_package(&dir, "json", "sifr-demo-json", "demo_json");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_json.parse import parse_json\n\n\
def main():\n    assert parse_json() == 1\n",
    );
    write_package_source(&json, "__init__.sifr", "from .parse import parse_json\n");
    write_package_source(
        &json,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &json],
        &[package_edge(&app, "demo_json", &json)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::PACKAGE_PRIVATE_MODULE_ACCESS.code()),
        "private dependency module should be rejected: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_reclassifies_unresolved_bare_stdlib_import() {
    let dir = mktemp_dir("package_bare_stdlib_import");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    write_package_source(
        &app,
        "main.sifr",
        "from math import sqrt\n\n\
def main():\n    pass\n",
    );
    let graph = package_graph(&dir, &[&app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    let diagnostic = errors
        .iter()
        .find(|error| error.code == DiagnosticCode::IMPORT_BARE_STDLIB.code())
        .unwrap_or_else(|| panic!("bare stdlib diagnostic should be emitted: {errors:?}"));
    assert_eq!(
        diagnostic.message,
        "bare stdlib import 'math'; Sifr stdlib lives under 'sifr.*'"
    );
    assert_eq!(diagnostic.args["bare_module"], "math".into());
    assert_eq!(diagnostic.args["suggested_module"], "sifr.math".into());
    assert_eq!(diagnostic.args["imported_names"], "sqrt".into());
    assert_eq!(
        diagnostic.help.as_deref(),
        Some("use 'from sifr.math import sqrt'")
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_rejects_transitive_dependency_import() {
    let dir = mktemp_dir("package_transitive_rejection");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let mid = production_package(&dir, "mid", "sifr-demo-mid", "demo_mid");
    let json = production_package(&dir, "json", "sifr-demo-json", "demo_json");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_json import parse_json\n\n\
def main():\n    assert parse_json() == 1\n",
    );
    write_package_source(&mid, "__init__.sifr", "from demo_json import parse_json\n");
    write_package_source(&json, "__init__.sifr", "from .parse import parse_json\n");
    write_package_source(
        &json,
        "parse.sifr",
        "def parse_json() -> int:\n    return 1\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &mid, &json],
        &[
            package_edge(&app, "demo_mid", &mid),
            package_edge(&mid, "demo_json", &json),
        ],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::PACKAGE_UNDECLARED_DIRECT_IMPORT.code()),
        "transitive dependency import should be rejected: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_package_project_reports_internal_reexport_cycles() {
    let dir = mktemp_dir("package_reexport_cycle");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let cycle = production_package(&dir, "cycle", "sifr-demo-cycle", "demo_cycle");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_cycle import value\n\n\
def main():\n    assert value() == 1\n",
    );
    write_package_source(&cycle, "__init__.sifr", "from .a import value\n");
    write_package_source(&cycle, "a.sifr", "from .b import value\n");
    write_package_source(&cycle, "b.sifr", "from .a import value\n");
    let graph = package_graph(
        &dir,
        &[&app, &cycle],
        &[package_edge(&app, "demo_cycle", &cycle)],
    );
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let entrypoint = package_entrypoint(&graph, &source_map, &app, app.root.join("src/main.sifr"));

    let errors = check_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new());

    assert!(
        errors
            .iter()
            .any(|error| error.code == DiagnosticCode::IMPORT_CYCLE.code()
                && error.spans.iter().any(|span| span.is_primary)
                && error.spans.iter().filter(|span| !span.is_primary).count() >= 2),
        "package re-export cycle should be reported: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
