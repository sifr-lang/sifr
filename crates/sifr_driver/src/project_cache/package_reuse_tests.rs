use super::*;
use sifr_frontend::{DiskSourceProvider, persistence::CapturedSource};
use std::{fs, path::PathBuf, process::Command};

fn compiler() -> sifr_identity::CompilerIdentity {
    sifr_identity::CompilerIdentity::for_test(crate::compiled_input_tokens(), "dxf-package")
}
fn inputs(package: &crate::PackageEntrypoint) -> SemanticInputs {
    SemanticInputs {
        compiler: compiler().as_str().into(),
        metadata: "metadata".into(),
        target: "test-target".into(),
        workspace_and_source_policy: "actual-package-owner".into(),
        package_and_lock: package_context::identity(package).unwrap(),
        language_options: "default".into(),
        diagnostic_policy: "canonical".into(),
        components: BTreeMap::new(),
        required_external: Default::default(),
        external: BTreeMap::new(),
    }
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"app\", \"dep\"]\nresolver = \"3\"\n",
    )
    .unwrap();
    for name in ["app", "dep"] {
        let dir = root.path().join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        let dependencies = if name == "app" {
            "\n[dependencies]\ndep = { path = \"../dep\", package = \"dxf-dep\" }\n"
        } else {
            ""
        };
        fs::write(dir.join("Cargo.toml"), format!(
            "[package]\nname = \"dxf-{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n{dependencies}"
        )).unwrap();
        fs::write(dir.join("sifr.toml"), format!(
            "[package]\nname = \"{name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n[source]\nroot = \"src\"\n"
        )).unwrap();
        fs::write(dir.join("src/lib.rs"), "").unwrap();
    }
    fs::write(
        root.path().join("app/src/main.sifr"),
        "from .middle import answer\ndef main() -> int:\n    return answer()\n",
    )
    .unwrap();
    fs::write(root.path().join("app/src/middle.sifr"),
        "from .helper import value\nfrom dep import value as other\ndef answer() -> int:\n    return value() + other()\n").unwrap();
    edit(root.path(), "app/src/helper.sifr", 1);
    edit(root.path(), "dep/src/__init__.sifr", 1);
    root
}
fn edit(root: &Path, path: &str, value: i32) {
    fs::write(
        root.join(path),
        format!("def value() -> int:\n    return {value}\n"),
    )
    .unwrap();
}
fn package(root: &Path) -> crate::PackageEntrypoint {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--offline"])
        .current_dir(root.join("app"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata =
        sifr_package::parse_metadata_json(&String::from_utf8_lossy(&output.stdout)).unwrap();
    let mut disk = DiskSourceProvider::new();
    let graph = sifr_package::derive_package_graph(metadata, &mut disk).unwrap();
    let source_map = sifr_package::PackageSourceMap::build(&graph, &mut disk).unwrap();
    let package_id = graph
        .packages
        .values()
        .find(|p| p.sifr_name.0 == "app")
        .unwrap()
        .package_id
        .clone();
    crate::PackageEntrypoint {
        main_file: root.join("app/src/main.sifr"),
        package_id,
        graph,
        source_map,
        python_runtime: None,
        lock_mode: sifr_package::CargoLockMode::Normal,
    }
}
fn fresh(
    package: &crate::PackageEntrypoint,
    provider: &mut dyn SourceProvider,
) -> Vec<RenderedDiagnostic> {
    let resolved = crate::project::parse_package_import_closure_source_project(
        &package.graph,
        &package.source_map,
        &package.package_id,
        &package.main_file,
        crate::project::DiscoveryDiagnosticStyle::ModuleName,
        provider,
    );
    match resolved.and_then(|mut parsed| {
        let entry = parsed
            .parsed_modules
            .remove(&parsed.entry_module_name)
            .unwrap();
        parsed.parsed_modules.insert("main".into(), entry);
        crate::project::collect_project_hir_source_modules(
            &parsed.parsed_modules,
            Default::default(),
        )
    }) {
        Ok(_) => Vec::new(),
        Err(errors) => errors,
    }
}
fn run(
    root: &Path,
    package: &crate::PackageEntrypoint,
    context: SemanticInputs,
) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    run_with_cache(root, &root.join("cache"), package, context)
}
fn run_with_cache(
    root: &Path,
    cache: &Path,
    package: &crate::PackageEntrypoint,
    context: SemanticInputs,
) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    check(
        (cache, &root.join("app")),
        &package.main_file,
        &mut DiskSourceProvider::new(),
        context,
        &AtomicBool::new(false),
        Some((&compiler(), &Default::default(), Some(package))),
        |provider| fresh(package, provider).into(),
    )
}
fn assert_reused(report: &ProjectCacheReport, changed: &Path) {
    assert_eq!(report.status, "interface-restored", "{report:?}");
    assert!(
        report
            .modules
            .iter()
            .any(|m| m.path == changed && m.action == "computed")
    );
    assert!(
        report
            .modules
            .iter()
            .any(|m| m.path.ends_with("middle.sifr") && m.action == "restored")
    );
    assert!(
        report
            .modules
            .iter()
            .any(|m| m.path.ends_with("main.sifr") && m.action == "restored")
    );
}

#[test]
fn package_cwd_body_edit_restores_importer() {
    let root = fixture();
    let entry = package(root.path());
    assert_eq!(
        run(root.path(), &entry, inputs(&entry)).1.status,
        "published"
    );
    for path in ["app/src/helper.sifr", "dep/src/__init__.sifr"] {
        edit(root.path(), path, 2);
        let (actual, report) = run(root.path(), &entry, inputs(&entry));
        assert!(actual.is_empty());
        assert_eq!(actual, fresh(&entry, &mut DiskSourceProvider::new()));
        assert_reused(&report, &root.path().join(path));
    }
}

#[test]
fn package_authority_changes_invalidate() {
    let root = fixture();
    let entry = package(root.path());
    assert!(run(root.path(), &entry, inputs(&entry)).0.is_empty());
    edit(root.path(), "app/src/helper.sifr", 2);
    for key in [
        "compiler",
        "metadata",
        "target",
        "source-policy",
        "lock",
        "trust",
        "dependency",
        "manifest",
    ] {
        let mut current = entry.clone();
        let mut context = inputs(&entry);
        match key {
            "compiler" => context.compiler.push_str("-changed"),
            "metadata" => context.metadata.push_str("-changed"),
            "target" => context.target.push_str("-changed"),
            "source-policy" => context.workspace_and_source_policy.push_str("-changed"),
            "lock" => current.lock_mode = sifr_package::CargoLockMode::Locked,
            "trust" => {
                current
                    .graph
                    .packages
                    .get_mut(&entry.package_id)
                    .unwrap()
                    .manifest
                    .trust
                    .security_capabilities
                    .push("new-capability".into());
            }
            "dependency" => {
                current
                    .graph
                    .packages
                    .values_mut()
                    .find(|p| p.sifr_name.0 == "dep")
                    .unwrap()
                    .cargo_version = "0.2.0".into();
            }
            "manifest" => {
                current
                    .graph
                    .packages
                    .get_mut(&entry.package_id)
                    .unwrap()
                    .manifest
                    .source_features
                    .insert("extra".into(), "module".into());
            }
            _ => unreachable!(),
        }
        if !matches!(key, "compiler" | "metadata" | "target" | "source-policy") {
            context = inputs(&current);
        }
        let (_, report) = run(root.path(), &current, context);
        assert_eq!(report.restored_checks, 0, "{key}: {report:?}");
    }
    for kind in ["python", "native", "sql"] {
        let mut current = entry.clone();
        let manifest = &mut current
            .graph
            .packages
            .get_mut(&entry.package_id)
            .unwrap()
            .manifest;
        match kind {
            "python" => manifest.python.requires_imports.push("live".into()),
            "native" => manifest.rust.direct_crate_bindings = true,
            "sql" => {
                manifest.sql.requirements.insert(
                    "live".into(),
                    sifr_package::SqlRequirementConfig {
                        capabilities: Default::default(),
                        providers: Default::default(),
                    },
                );
            }
            _ => unreachable!(),
        }
        assert!(package_context::identity(&current).is_none(), "{kind}");
    }
    // Current source ownership and resolver failures cannot be bypassed by the
    // old success record, even if a caller retains its old semantic token.
    let mut missing = entry.clone();
    missing
        .source_map
        .modules
        .retain(|_, m| !m.file_path.ends_with("helper.sifr"));
    let (actual, report) = run(root.path(), &missing, inputs(&entry));
    assert!(!actual.is_empty());
    assert_eq!(report.restored_checks, 0);
    assert_eq!(actual, fresh(&missing, &mut DiskSourceProvider::new()));
}

#[test]
fn package_chained_restore_matches_fresh() {
    let root = fixture();
    let entry = package(root.path());
    assert!(run(root.path(), &entry, inputs(&entry)).0.is_empty());
    for value in 2..7 {
        edit(root.path(), "app/src/helper.sifr", value);
        let (actual, report) = run(root.path(), &entry, inputs(&entry));
        assert_reused(&report, &root.path().join("app/src/helper.sifr"));
        assert_eq!(actual, fresh(&entry, &mut DiskSourceProvider::new()));
        // The next hop must carry current source bytes, rather than claiming
        // success using only the original ancestor's erased body.
        let store = storage::Store::open(
            &root.path().join("cache"),
            &root.path().join("app"),
            &inputs(&entry).identity().unwrap(),
        )
        .unwrap();
        let generation = store.latest().unwrap();
        let current = CapturedSource {
            path: root.path().join("app/src/helper.sifr"),
            text: fs::read_to_string(root.path().join("app/src/helper.sifr")).unwrap(),
        };
        assert!(generation.records().flatten().any(|record| {
            record
                .result
                .resolution
                .ready()
                .is_some_and(|r| r.sources.contains(&current))
                && record.validate(&inputs(&entry), &mut DiskSourceProvider::new())
        }));
    }
    for (before, after, error) in [
        (
            "def value(x: int = 1) -> int:\n    return x\n",
            "def value(x: int = 2) -> int:\n    return x\n",
            false,
        ),
        (
            "N: int = 1\ndef value() -> int:\n    return N\n",
            "N: int = 2\ndef value() -> int:\n    return N\n",
            false,
        ),
        (
            "def value() -> int:\n    return 1\n",
            "def value() -> str:\n    return \"bad\"\n",
            true,
        ),
        (
            "def value() -> int:\n    return 1\n",
            "def value() -> int:\n    return \"bad\"\n",
            true,
        ),
    ] {
        let path: PathBuf = root.path().join("app/src/helper.sifr");
        fs::write(&path, before).unwrap();
        assert!(run(root.path(), &entry, inputs(&entry)).0.is_empty());
        fs::write(&path, after).unwrap();
        let (actual, report) = run(root.path(), &entry, inputs(&entry));
        assert_ne!(report.status, "interface-restored");
        assert_eq!(!actual.is_empty(), error);
        assert_eq!(actual, fresh(&entry, &mut DiskSourceProvider::new()));
    }
}

// Each test owns its package graph and cache history. Visibility is resolved by
// the real package source map, including an existing private implementation.
fn assert_private_import_rejected(private_import: &str) {
    let root = fixture();
    edit(root.path(), "dep/src/hidden.sifr", 7);
    let entry = package(root.path());
    let middle = root.path().join("app/src/middle.sifr");
    let public_source = fs::read_to_string(&middle).unwrap();
    let (initial, report) = run(root.path(), &entry, inputs(&entry));
    assert!(initial.is_empty());
    assert_eq!(report.status, "published");

    let helper = root.path().join("app/src/helper.sifr");
    edit(root.path(), "app/src/helper.sifr", 2);
    let (warm, report) = run(root.path(), &entry, inputs(&entry));
    assert!(warm.is_empty());
    assert_reused(&report, &helper);

    fs::write(
        &middle,
        public_source.replace("from dep import value as other", private_import),
    )
    .unwrap();
    // Rebuild from Cargo metadata at the real app cwd, as a new CLI request
    // does. No hand-built graph/token can conceal current resolver authority.
    let private_entry = package(root.path());
    let expected = fresh(&private_entry, &mut DiskSourceProvider::new());
    assert_eq!(expected.len(), 1, "{expected:?}");
    assert_eq!(expected[0].code, "SIFR-PACKAGE-0203");
    assert!(expected[0].message.contains("private module 'dep.hidden'"));
    let (actual, report) = run(root.path(), &private_entry, inputs(&private_entry));
    assert_eq!(actual, expected);
    assert_eq!(report.restored_checks, 0, "{report:?}");
    assert_ne!(report.status, "restored", "{report:?}");
    assert_ne!(report.status, "interface-restored", "{report:?}");

    let cold_cache = root.path().join("independent-cold-cache");
    assert!(!cold_cache.exists());
    let (cold, cold_report) = run_with_cache(
        root.path(),
        &cold_cache,
        &private_entry,
        inputs(&private_entry),
    );
    assert_eq!(cold, expected);
    assert_eq!(cold_report.restored_checks, 0, "{cold_report:?}");

    fs::write(&middle, public_source).unwrap();
    let recovered = package(root.path());
    let (actual, _) = run(root.path(), &recovered, inputs(&recovered));
    assert!(actual.is_empty());
    assert_eq!(actual, fresh(&recovered, &mut DiskSourceProvider::new()));
    edit(root.path(), "app/src/helper.sifr", 3);
    let (actual, report) = run(root.path(), &recovered, inputs(&recovered));
    assert!(actual.is_empty());
    assert_eq!(actual, fresh(&recovered, &mut DiskSourceProvider::new()));
    assert_reused(&report, &helper);
}

#[test]
fn package_private_symbol_edit_rejects_restored_success() {
    assert_private_import_rejected("from dep.hidden import value as other");
}
