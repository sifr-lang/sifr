use super::*;
use std::path::{Path, PathBuf};

const BIIP_MAIN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../verification/areas/python_interop/fixtures/package_bridge_archive/main.sifr"
));
const BIIP_BRIDGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../verification/areas/python_interop/fixtures/package_bridge_archive/identifiers.py"
));

#[test]
#[cfg(unix)]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn archived_biip_bridge_builds_and_runs_without_checkout_or_extraction() {
    use std::os::unix::fs::PermissionsExt as _;

    let dir = mktemp_dir("archived_biip_python_bridge");
    let app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    write_package_source(&app, "main.sifr", BIIP_MAIN);
    write_package_source(&app, "python_bridges/identifiers.py", BIIP_BRIDGE);
    append_python_trust(&app, &["biip"]);

    let source_graph = package_graph(&dir, &[&app], &[]);
    let package = source_graph
        .packages
        .values()
        .next()
        .expect("source package metadata");
    let inventory = sifr_package::discover_python_bridge_inventory(package)
        .expect("bridge inventory should be generated from package inputs");
    sifr_package::write_python_bridge_inventory(package, &inventory)
        .expect("bridge inventory should be written");
    let installed_app = package_and_unpack(&dir, &app);
    assert!(
        installed_app
            .root
            .join("src/python_bridges/__sifr_inventory__.json")
            .is_file()
    );
    std::fs::remove_dir_all(&app.root).expect("source checkout should be removed before build");

    let graph = package_graph(&dir, &[&installed_app], &[]);
    let source_map = sifr_package::PackageSourceMap::build(
        &graph,
        &mut sifr_frontend::DiskSourceProvider::new(),
    )
    .expect("source map builds");
    let mut entrypoint = package_entrypoint(
        &graph,
        &source_map,
        &installed_app,
        installed_app.root.join("src/main.sifr"),
    );
    entrypoint.python_runtime = Some(verification_python_runtime(&["biip"]));
    let artifact =
        build_cached_package_project(&entrypoint, &mut sifr_frontend::DiskSourceProvider::new())
            .expect("installed biip bridge binary should build from archived source");
    std::fs::remove_dir_all(installed_app.root.join("src/python_bridges"))
        .expect("installed bridge sources should be removable after build");

    let run_root = dir.join("read-only-run-root");
    std::fs::create_dir_all(&run_root).expect("run root should be created");
    std::fs::set_permissions(&run_root, std::fs::Permissions::from_mode(0o555))
        .expect("run root should become read-only");
    let output = std::process::Command::new(artifact.binary_path())
        .current_dir(&run_root)
        .env("TMPDIR", &run_root)
        .output()
        .expect("installed biip bridge binary should run");

    assert!(
        output.status.success(),
        "binary should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "sifr-python-interop:package-bridge:gtin=7032069804988:format=13:check=8"
    );
    if let Some(marker_file) = std::env::var_os("SIFR_PACKAGE_BRIDGE_DEMO_MARKER_FILE") {
        std::fs::write(marker_file, &output.stdout)
            .expect("demo marker file should capture compiled binary output");
    }
    assert_eq!(
        std::fs::read_dir(&run_root)
            .expect("run root should be readable")
            .count(),
        0,
        "embedded bridge loading must not extract files"
    );
    std::fs::set_permissions(&run_root, std::fs::Permissions::from_mode(0o755))
        .expect("run root should become removable");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn two_packages_can_execute_the_same_bridge_module_path_without_collision() {
    let dir = mktemp_dir("two_package_python_bridge_isolation");
    let mut app = production_package(&dir, "app", "sifr-demo-app", "demo_app");
    let library = production_package(&dir, "library_dep", "library_dep", "demo_library");
    app.aliases.push(TestAlias {
        alias: "demo_library".to_string(),
        dependency: "library_dep".to_string(),
        import: "demo_library".to_string(),
    });
    write_manifest_dependency_alias(&app, "library_dep", "demo_library");
    write_package_source(
        &app,
        "main.sifr",
        "from demo_library import library_value\nfrom sifr.python import PythonError\n\n@python(bridge.identifiers.value)\ndef app_value() -> Result[int, PythonError]: ...\n\n@blocking_io\ndef main() -> Result[None, PythonError]:\n    try:\n        own: int = app_value()\n        dependency: int = library_value()\n        assert own == 40\n        assert dependency == 2\n        print(\"sifr-python-interop:package-bridge:isolation=42\")\n    except PythonError as error:\n        raise error\n    return None\n",
    );
    write_package_source(
        &library,
        "__init__.sifr",
        "from sifr.python import PythonError\n\n@python(bridge.identifiers.value)\ndef bridge_value() -> Result[int, PythonError]: ...\n\ndef library_value() -> int:\n    try:\n        value: int = bridge_value()\n        return value\n    except PythonError:\n        return -1\n",
    );
    write_package_source(
        &app,
        "python_bridges/identifiers.py",
        "def value():\n    return 40\n",
    );
    write_package_source(
        &library,
        "python_bridges/identifiers.py",
        "def value():\n    return 2\n",
    );
    let graph = package_graph(
        &dir,
        &[&app, &library],
        &[package_edge(&app, "library_dep", &library)],
    );
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
            .expect("two-package bridge binary should build");
    std::fs::remove_dir_all(app.root.join("src/python_bridges"))
        .expect("app bridge checkout should be removable");
    std::fs::remove_dir_all(library.root.join("src/python_bridges"))
        .expect("dependency bridge checkout should be removable");
    let output = std::process::Command::new(artifact.binary_path())
        .output()
        .expect("two-package bridge binary should run");

    assert!(
        output.status.success(),
        "binary should pass: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "sifr-python-interop:package-bridge:isolation=42"
    );
    let _ = std::fs::remove_dir_all(dir);
}

fn append_python_trust(package: &TestPackage, roots: &[&str]) {
    let manifest = package.root.join("sifr.toml");
    let mut source = std::fs::read_to_string(&manifest).expect("manifest should be readable");
    let roots = roots
        .iter()
        .map(|root| format!("\"{root}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = write!(source, "\n[trust]\npython = [{roots}]\n");
    std::fs::write(manifest, source).expect("manifest trust should be updated");
}

fn package_and_unpack(workspace: &Path, app: &TestPackage) -> TestPackage {
    let packaged = std::process::Command::new("cargo")
        .args(["package", "--allow-dirty", "--no-verify", "--manifest-path"])
        .arg(app.root.join("Cargo.toml"))
        .output()
        .expect("cargo package should run");
    assert!(
        packaged.status.success(),
        "cargo package should pass: {}",
        String::from_utf8_lossy(&packaged.stderr)
    );
    let install_root = workspace.join("installed");
    std::fs::create_dir_all(&install_root).expect("install root should be created");
    let archive = app.root.join(format!(
        "target/package/{}-{}.crate",
        app.cargo_name, app.version
    ));
    let unpacked = std::process::Command::new("tar")
        .args(["-xzf"])
        .arg(&archive)
        .arg("-C")
        .arg(&install_root)
        .output()
        .expect("package archive should unpack");
    assert!(unpacked.status.success(), "package archive should unpack");
    let mut installed = app.clone();
    installed.root = install_root.join(format!("{}-{}", app.cargo_name, app.version));
    installed
}

fn verification_python_runtime(roots: &[&str]) -> crate::PackagePythonRuntime {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let area_root = repo_root.join("verification/areas/python_interop");
    let venv_root = area_root.join(".venv");
    let interpreter = if cfg!(windows) {
        venv_root.join("Scripts/python.exe")
    } else {
        venv_root.join("bin/python")
    };
    let roots = roots
        .iter()
        .map(|root| (*root).to_string())
        .collect::<Vec<_>>();
    let request = sifr_package::PythonEnvironmentProbeRequest {
        venv_root,
        interpreter,
        pyproject: Some(area_root.join("pyproject.toml")),
        lock: Some(area_root.join("uv.lock")),
        required_imports: roots.clone(),
        declared_imports: roots.clone(),
        native_imports: Vec::new(),
    };
    let probe = sifr_package::probe_python_environment(&request)
        .expect("locked Python interop environment should probe");
    let digest = sifr_package::digest_python_environment_probe(&request, &probe)
        .expect("test Python environment identity should serialize")
        .hex;
    crate::PackagePythonRuntime::from_probe(
        &request,
        &probe,
        digest,
        roots.clone(),
        roots,
        Vec::new(),
    )
    .expect("test Python authoring identity should serialize")
}
