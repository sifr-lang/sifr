use super::*;
use sifr_frontend::{
    DiskSourceProvider, FrontendContext, FrontendInput, FrontendMode, SourcePath, SourceText,
};
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{
    fs,
    process::{Child, Command},
    thread,
    time::Duration,
};

pub(super) fn context() -> SemanticInputs {
    SemanticInputs {
        compiler: "compiler-fixture".into(),
        metadata: "real-frontend-no-stdlib".into(),
        target: "host".into(),
        workspace_and_source_policy: "saved-source-v1".into(),
        package_and_lock: "manifestless".into(),
        language_options: "default".into(),
        diagnostic_policy: "canonical".into(),
        components: BTreeMap::new(),
        required_external: Default::default(),
        external: BTreeMap::new(),
    }
}
pub(super) fn compute(file: &Path, provider: &mut dyn SourceProvider) -> Vec<RenderedDiagnostic> {
    let source = provider.read_file(file).unwrap();
    let mut frontend = FrontendContext::load_single_file(FrontendInput {
        path: SourcePath::new(file),
        source,
        mode: FrontendMode::SingleFile,
    })
    .unwrap();
    frontend.diagnostics_for_project().into_value().diagnostics
}
pub(super) fn run(cache: &Path, file: &Path) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    check(
        (cache, file.parent().unwrap()),
        file,
        &mut DiskSourceProvider::new(),
        context(),
        &AtomicBool::new(false),
        None,
        |provider| compute(file, provider).into(),
    )
}
pub(super) fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    let file = workspace.join("main.sifr");
    fs::write(&file, "def main() -> None:\n    value: int = 1\n").unwrap();
    let cache = root.path().join("cache");
    (root, file, cache)
}
pub(super) fn store(cache: &Path, file: &Path) -> storage::Store {
    storage::Store::open(
        cache,
        file.parent().unwrap(),
        &context().identity().unwrap(),
    )
    .unwrap()
}
#[test]
fn completed_families() {
    let (_root, file, cache) = fixture();
    let (fresh, first) = run(&cache, &file);
    assert_eq!(first.status, "published");
    let (restored, second) = run(&cache, &file);
    assert_eq!(fresh, restored);
    assert_eq!(second.restored_checks, 1);
    let generation = store(&cache, &file).latest().unwrap();
    let record = generation.records().next().unwrap().unwrap();
    assert!(record.result.check_complete());
    assert!(!record.result.codegen_complete("any"));
    assert!(record.result.checked.ready().is_none());
    // A restored shallow check leaves typed/editor queries to normal computation.
    let source = fs::read_to_string(&file).unwrap();
    let mut frontend = FrontendContext::load_single_file(FrontendInput {
        path: SourcePath::new(&file),
        source: SourceText::new(source.clone()),
        mode: FrontendMode::SingleFile,
    })
    .unwrap();
    let module = frontend.module_graph().entrypoint;
    assert_eq!(
        frontend.hir_module_view(module).metadata().cache_status,
        sifr_frontend::CacheStatus::Miss
    );
    fs::write(&file, "def main() -> None:\n    x: int = \"bad\"\n").unwrap();
    let (errors, changed) = run(&cache, &file);
    assert!(!errors.is_empty());
    assert_eq!(changed.status, "published");
    let (again, hit) = run(&cache, &file);
    assert_eq!(again, errors);
    assert_eq!(hit.restored_checks, 1);
    fs::write(&file, source).unwrap();
    let (reverted, hit) = run(&cache, &file);
    assert_eq!(reverted, fresh);
    assert_eq!(hit.restored_checks, 1);
    // SourceMap allocation history does not appear in canonical payloads.
    let mut map = sifr_diagnostics::SourceMap::default();
    map.register_source("unrelated", "padding");
    let generation = store(&cache, &file).latest().unwrap();
    for record in generation.records() {
        let record = record.unwrap();
        let sources = record
            .result
            .resolution
            .ready()
            .unwrap()
            .sources
            .iter()
            .map(|source| {
                let identity = source.identity();
                let display = record.displays.get(&identity).unwrap().clone();
                (identity, (source.clone(), display))
            })
            .collect();
        let remapped: Vec<_> = record
            .result
            .diagnostics
            .ready()
            .unwrap()
            .iter()
            .map(|diagnostic| diagnostic.render(&sources, &mut map).unwrap())
            .collect();
        assert_eq!(remapped, record.diagnostics().unwrap());
    }
}
#[cfg(unix)]
#[test]
fn dx13_c04_unavailable_readonly_and_hintless() {
    let (_root, file, cache) = fixture();
    fs::write(&cache, "blocked cache root").unwrap();
    let (fresh, unavailable) = run(&cache, &file);
    assert_eq!(unavailable.status, "unavailable");
    fs::remove_file(&cache).unwrap();
    let (ready, report) = run(&cache, &file);
    assert_eq!(report.status, "published");
    assert_eq!(fresh, ready);
    fs::remove_file(file.parent().unwrap().join(".sifrbuildinfo")).unwrap();
    fs::set_permissions(file.parent().unwrap(), fs::Permissions::from_mode(0o500)).unwrap();
    let (again, restored) = run(&cache, &file);
    assert_eq!(restored.restored_checks, 1);
    assert_eq!(again, fresh);
    fs::set_permissions(file.parent().unwrap(), fs::Permissions::from_mode(0o700)).unwrap();
    let store = store(&cache, &file);
    fs::set_permissions(&store.root, fs::Permissions::from_mode(0o500)).unwrap();
    fs::write(&file, "def main() -> None:\n    x: int = 2\n").unwrap();
    let (_, report) = run(&cache, &file);
    assert_eq!(report.status, "write-unavailable");
    fs::set_permissions(&store.root, fs::Permissions::from_mode(0o700)).unwrap();
}
#[cfg(unix)]
#[test]
fn dx13_c08_c09_inherited_payloads_reader_gc() {
    let (_root, file, cache) = fixture();
    run(&cache, &file);
    let store = store(&cache, &file);
    let old = store.latest().unwrap();
    let old_record = old.manifest.records.first().unwrap().clone();
    let before = fs::read(old.path.join(&old_record)).unwrap();
    let other = file.parent().unwrap().join("other.sifr");
    fs::write(&other, "def main() -> None:\n    missing()\n").unwrap();
    let (_, published) = run(&cache, &other);
    assert_eq!(published.status, "published");
    let latest = store.latest().unwrap();
    assert_eq!(latest.manifest.records.len(), 2);
    assert_eq!(
        fs::metadata(old.path.join(&old_record)).unwrap().ino(),
        fs::metadata(latest.path.join(&old_record)).unwrap().ino()
    );
    assert_eq!(fs::read(old.path.join(&old_record)).unwrap(), before);
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
    assert!(old.path.exists());
    let mut unrelated = Command::new("sleep").arg("10").spawn().unwrap();
    drop(old);
    assert_eq!(store.prune(false, false).unwrap().deleted_generations, 0);
    assert_eq!(store.prune(true, true).unwrap().eligible_generations, 1);
    let removed = store.prune(true, false).unwrap().deleted_generations;
    unrelated.kill().unwrap();
    unrelated.wait().unwrap();
    assert_eq!(removed, 1);
    let (_, reused) = run(&cache, &file);
    assert_eq!(reused.restored_checks, 1);
}
#[test]
fn dx13_c09_corrupt_schema_references_and_winners() {
    let (_root, file, cache) = fixture();
    run(&cache, &file);
    let store = store(&cache, &file);
    let generation = store.latest().unwrap();
    let record = generation.manifest.records.first().unwrap();
    fs::write(generation.path.join(record), "incomplete").unwrap();
    let (actual, report) = run(&cache, &file);
    assert_eq!(report.computed_checks, 1);
    assert_eq!(report.restored_checks, 0);
    assert_eq!(actual, compute(&file, &mut DiskSourceProvider::new()));
    // The corrupt winner is not trusted or rewritten in place.
    assert_eq!(
        fs::read(generation.path.join(record)).unwrap(),
        b"incomplete"
    );
    let mut record = super_record(&file);
    record.schema += 1;
    assert!(!record.validate(&context(), &mut DiskSourceProvider::new()));
    let mut unknown = serde_json::to_value(super_record(&file)).unwrap();
    unknown["result"]["diagnostics"]["Complete"] = serde_json::json!([]);
    unknown["result"]["resolution"]["Complete"]["extra"] = true.into();
    assert!(serde_json::from_value::<CompletedCheck>(unknown).is_err());
}
pub(super) fn super_record(file: &Path) -> CompletedCheck {
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    let result = compute(file, &mut capture);
    CompletedCheck::capture(file, context(), &capture, &result).unwrap()
}
#[test]
fn cancel_transient_and_changed_input() {
    let (_root, file, cache) = fixture();
    let cancel = AtomicBool::new(false);
    let (_, report) = check(
        (&cache, file.parent().unwrap()),
        &file,
        &mut DiskSourceProvider::new(),
        context(),
        &cancel,
        None,
        |provider| {
            let result = compute(&file, provider);
            cancel.store(true, Ordering::Release);
            result.into()
        },
    );
    assert_eq!(report.status, "cancelled");
    assert!(store(&cache, &file).latest().is_none());
    let (_, report) = check(
        (&cache, file.parent().unwrap()),
        &file,
        &mut DiskSourceProvider::new(),
        context(),
        &AtomicBool::new(false),
        None,
        |provider| {
            let result = compute(&file, provider);
            fs::write(&file, "def main() -> None:\n    pass\n").unwrap();
            result.into()
        },
    );
    assert_eq!(report.status, "changed-inputs");
    let (_, report) = check(
        (&cache, file.parent().unwrap()),
        &file,
        &mut DiskSourceProvider::new(),
        context(),
        &AtomicBool::new(false),
        None,
        |provider| {
            provider.read_file(&file).unwrap();
            vec![crate::diagnostics::diagnostic_with_code(
                "temporary environment failure",
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
            .into()
        },
    );
    assert_eq!(report.status, "uncacheable");
    assert!(store(&cache, &file).latest().is_none());
}

pub(super) fn pause(point: &str) {
    if std::env::var("SIFR_DX13_PAUSE").ok().as_deref() == Some(point) {
        let marker = std::env::var("SIFR_DX13_MARKER").unwrap();
        fs::write(marker, point).unwrap();
        loop {
            thread::sleep(Duration::from_millis(20));
        }
    }
}
pub(super) fn worker(file: &Path, cache: &Path, mode: &str, marker: &Path) -> Child {
    Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "project_cache::tests::dx13_process_worker",
            "--nocapture",
        ])
        .env("SIFR_DX13_FILE", file)
        .env("SIFR_DX13_CACHE", cache)
        .env("SIFR_DX13_PAUSE", mode)
        .env("SIFR_DX13_MARKER", marker)
        .spawn()
        .unwrap()
}
#[test]
fn dx13_process_worker() {
    let Ok(file) = std::env::var("SIFR_DX13_FILE") else {
        return;
    };
    let cache = std::env::var("SIFR_DX13_CACHE").unwrap();
    if std::env::var("SIFR_DX13_PAUSE").ok().as_deref() == Some("reader") {
        let store = store(Path::new(&cache), Path::new(&file));
        let generation = store.latest().unwrap();
        assert!(
            generation
                .records()
                .all(|record| record.unwrap().diagnostics().is_ok())
        );
        pause("reader");
    }
    let (diagnostics, report) = run(Path::new(&cache), Path::new(&file));
    let output = serde_json::json!({"diagnostics": diagnostics, "report": report});
    fs::write(
        std::env::var("SIFR_DX13_MARKER").unwrap(),
        output.to_string(),
    )
    .unwrap();
}
#[test]
fn process_death_and_new_process_restore() {
    let (root, file, cache) = fixture();
    for point in ["before-rename", "after-rename"] {
        let marker = root.path().join(point);
        let mut child = worker(&file, &cache, point, &marker);
        let deadline = Instant::now() + Duration::from_secs(15);
        while !marker.exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(marker.exists());
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(store(&cache, &file).latest().is_none());
        store(&cache, &file).prune(true, false).unwrap();
    }
    let first = root.path().join("first.json");
    assert!(
        worker(&file, &cache, "none", &first)
            .wait()
            .unwrap()
            .success()
    );
    let second = root.path().join("second.json");
    assert!(
        worker(&file, &cache, "none", &second)
            .wait()
            .unwrap()
            .success()
    );
    let fresh: serde_json::Value = serde_json::from_slice(&fs::read(first).unwrap()).unwrap();
    let restored: serde_json::Value = serde_json::from_slice(&fs::read(second).unwrap()).unwrap();
    assert_eq!(fresh["diagnostics"], restored["diagnostics"]);
    assert_eq!(fresh["report"]["computed_checks"], 1);
    assert_eq!(restored["report"]["restored_checks"], 1);
}

#[test]
fn dx13_c04_full_storage_preserves_source_outcome() {
    let (root, file, cache) = fixture();
    let marker = root.path().join("full.json");
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "project_cache::tests::dx13_process_worker",
            "--nocapture",
        ])
        .env("SIFR_DX13_FILE", &file)
        .env("SIFR_DX13_CACHE", &cache)
        .env("SIFR_DX13_STORAGE_FULL", "1")
        .env("SIFR_DX13_MARKER", &marker)
        .status()
        .unwrap();
    assert!(status.success());
    let full: serde_json::Value = serde_json::from_slice(&fs::read(marker).unwrap()).unwrap();
    assert_eq!(full["report"]["status"], "write-unavailable");
    assert_eq!(
        full["diagnostics"],
        serde_json::to_value(compute(&file, &mut DiskSourceProvider::new())).unwrap()
    );
    assert!(store(&cache, &file).latest().is_none());
    assert_eq!(run(&cache, &file).1.status, "published");
}
#[test]
fn dx13_c09_concurrent_process_reader_and_gc() {
    let (root, file, cache) = fixture();
    run(&cache, &file);
    let store = store(&cache, &file);
    let old = store.latest().unwrap().path.clone();
    let marker = root.path().join("reader");
    let mut reader = worker(&file, &cache, "reader", &marker);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(marker.exists());
    fs::write(&file, "def main() -> None:\n    absent()\n").unwrap();
    assert_eq!(run(&cache, &file).1.status, "published");
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
    assert!(old.exists());
    reader.kill().unwrap();
    reader.wait().unwrap();
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 1);
    assert!(!old.exists());
    assert_eq!(run(&cache, &file).1.restored_checks, 1);
}
#[cfg(unix)]
#[test]
fn dx13_c04_readonly_workspace_publishes_without_hint() {
    let (_root, file, cache) = fixture();
    fs::set_permissions(file.parent().unwrap(), fs::Permissions::from_mode(0o500)).unwrap();
    let (_, report) = run(&cache, &file);
    assert_eq!(report.status, "published");
    assert!(!file.parent().unwrap().join(".sifrbuildinfo").exists());
    assert_eq!(run(&cache, &file).1.restored_checks, 1);
    fs::set_permissions(file.parent().unwrap(), fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn dx13_resolved_package_context_and_live_external_inventory() {
    use sifr_package::{
        CargoLockMode, CargoPackageId, PackageSourceMap, SifrManifest, SifrPackageGraph,
        SifrPackageId, SifrPackageMetadata,
    };
    let (_root, file, _cache) = fixture();
    let package_id = SifrPackageId("pure-fixture".into());
    let cargo_package_id = CargoPackageId("pure-fixture".into());
    let manifest_path = file.parent().unwrap().join("sifr.toml");
    let manifest = SifrManifest::parse(
        &cargo_package_id,
        &manifest_path,
        "[package]\nname = \"pure\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n",
    )
    .unwrap();
    let package = SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id,
        cargo_package_name: "sifr-pure".into(),
        cargo_version: "0.1.0".into(),
        cargo_source: None,
        package_root: file.parent().unwrap().into(),
        sifr_manifest: manifest_path,
        sifr_name: manifest.package_name.clone(),
        manifest,
        aliases: BTreeMap::new(),
    };
    let mut entry = crate::PackageEntrypoint {
        main_file: file,
        package_id: package_id.clone(),
        graph: SifrPackageGraph {
            packages: BTreeMap::from([(package_id.clone(), package)]),
            cargo_edges: BTreeMap::new(),
            direct_dependency_scopes: BTreeMap::new(),
            backend_crates: BTreeMap::new(),
            classifications: BTreeMap::new(),
        },
        source_map: PackageSourceMap::default(),
        python_runtime: None,
        lock_mode: CargoLockMode::Normal,
    };
    let first = package_context::identity(&entry).unwrap().unwrap();
    entry
        .graph
        .packages
        .get_mut(&package_id)
        .unwrap()
        .manifest
        .source_features
        .insert("feature".into(), "module".into());
    assert_ne!(package_context::identity(&entry).unwrap().unwrap(), first);
    entry
        .graph
        .packages
        .get_mut(&package_id)
        .unwrap()
        .manifest
        .python
        .requires_imports
        .push("live_environment".into());
    assert!(package_context::identity(&entry).unwrap().is_none());
    entry
        .graph
        .packages
        .get_mut(&package_id)
        .unwrap()
        .manifest
        .python = Default::default();
    entry
        .graph
        .packages
        .get_mut(&package_id)
        .unwrap()
        .manifest
        .rust
        .direct_crate_bindings = true;
    assert!(package_context::identity(&entry).unwrap().is_none());
}

#[test]
fn completed_live_operation_stays_uncached() {
    let (_root, file, cache) = fixture();
    let (_, report) = check(
        (&cache, file.parent().unwrap()),
        &file,
        &mut DiskSourceProvider::new(),
        context(),
        &AtomicBool::new(false),
        None,
        |provider| CheckComputation {
            diagnostics: compute(&file, provider),
            reusable: false,
        },
    );
    assert_eq!(report.status, "external-context");
    assert!(store(&cache, &file).latest().is_none());
}

#[test]
fn moved_workspace_misses_without_changing_diagnostics() {
    let (root, file, cache) = fixture();
    for source in [
        "def main() -> None:\n    value: int = 1\n",
        "def main() -> None:\n    value: int = \"bad\"\n",
    ] {
        fs::write(&file, source).unwrap();
        let (_, original) = run(&cache, &file);
        assert_eq!(original.computed_checks, 1);
        assert_eq!(run(&cache, &file).1.restored_checks, 1);
        let moved_root = root.path().join("moved");
        fs::rename(file.parent().unwrap(), &moved_root).unwrap();
        let moved_file = moved_root.join("main.sifr");
        assert_eq!(fs::read_to_string(&moved_file).unwrap(), source);
        // The hint moves too, but its old canonical owner cannot authorize reuse.
        let (diagnostics, moved) = run(&cache, &moved_file);
        assert_eq!(moved.restored_checks, 0);
        assert_eq!(moved.computed_checks, 1);
        assert_eq!(
            diagnostics,
            compute(&moved_file, &mut DiskSourceProvider::new())
        );
        assert_eq!(diagnostics.is_empty(), !source.contains("bad"));
        let (again, warm) = run(&cache, &moved_file);
        assert_eq!(again, diagnostics);
        assert_eq!(warm.restored_checks, 1);
        fs::rename(&moved_root, file.parent().unwrap()).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn saved_check_policy_distinguishes_missing_empty_and_unserializable_paths() {
    use std::os::unix::ffi::OsStringExt;

    let cwd = Path::new("/workspace");
    let absent = saved_check_policy(Path::new(""), cwd).unwrap();
    let empty = saved_check_policy(Path::new("main.sifr"), cwd).unwrap();
    assert_ne!(absent, empty);
    let invalid = std::ffi::OsString::from_vec(b"/invalid-\xff/main.sifr".to_vec());
    let error = saved_check_policy(Path::new(&invalid), cwd).unwrap_err();
    assert!(error.contains("could not serialize saved-check policy"));
}

#[cfg(unix)]
#[test]
fn non_utf8_record_serialization_disables_publication() {
    use std::os::unix::ffi::OsStringExt;

    let (_root, file, cache) = fixture();
    let invalid_name = std::ffi::OsString::from_vec(b"invalid-\xff.sifr".to_vec());
    let invalid_file = file.with_file_name(invalid_name);
    struct Synthetic;
    impl SourceProvider for Synthetic {
        fn read_file(
            &mut self,
            _path: &Path,
        ) -> Result<SourceText, sifr_frontend::SourceProviderError> {
            Ok(SourceText::new("def main() -> None:\n    pass\n"))
        }
        fn read_dir(
            &mut self,
            _path: &Path,
        ) -> Result<Vec<sifr_frontend::SourceDirEntry>, sifr_frontend::SourceProviderError>
        {
            Ok(Vec::new())
        }
        fn is_file(&mut self, _path: &Path) -> bool {
            true
        }
        fn is_dir(&mut self, _path: &Path) -> bool {
            false
        }
        fn canonicalize(
            &mut self,
            path: &Path,
        ) -> Result<std::path::PathBuf, sifr_frontend::SourceProviderError> {
            Ok(path.to_path_buf())
        }
    }
    let (diagnostics, report) = check(
        (&cache, file.parent().unwrap()),
        &invalid_file,
        &mut Synthetic,
        context(),
        &AtomicBool::new(false),
        None,
        |provider| {
            provider.read_file(&invalid_file).unwrap();
            CheckComputation {
                diagnostics: Vec::new(),
                reusable: true,
            }
        },
    );
    assert!(diagnostics.is_empty());
    assert_eq!(report.status, "serialization-unavailable");
    assert_eq!(report.payload_bytes, 0);
    assert!(store(&cache, &file).latest().is_none());
}

#[cfg(windows)]
#[test]
fn windows_portability_concurrent_writer_gc_abandoned_stage_and_winner() {
    let (root, file, cache) = fixture();
    let marker = root.path().join("before-rename");
    let mut writer = worker(&file, &cache, "before-rename", &marker);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(marker.exists(), "writer did not stage a generation");
    let store = store(&cache, &file);
    assert!(
        store.prune(true, false).is_err(),
        "GC ignored the live writer lease"
    );
    writer.kill().unwrap();
    writer.wait().unwrap();
    store.prune(true, false).unwrap();
    assert!(store.latest().is_none());
    assert!(
        fs::read_dir(store.root.join("generations"))
            .unwrap()
            .all(|entry| !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".stage-")),
        "abandoned generation stage survived GC"
    );
    assert_eq!(run(&cache, &file).1.status, "published");
    let winner = store.latest().unwrap();
    assert!(winner.records().all(|record| record.is_ok()));
    assert_eq!(store.prune(true, false).unwrap().deleted_generations, 0);
}

#[cfg(windows)]
#[test]
fn windows_portability_workspace_identity_alias_and_orphan_prune() {
    let (root, file, cache) = fixture();
    assert_eq!(run(&cache, &file).1.status, "published");
    let store = store(&cache, &file);
    let context = store.root.clone();
    let workspace = file.parent().unwrap().to_path_buf();
    let outside = root.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("keep"), "intact").unwrap();

    let alias = context.join("latest.stage-10-20");
    let status = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&alias)
        .arg(&outside)
        .status()
        .unwrap();
    assert!(status.success());
    store.prune(true, false).unwrap();
    assert_eq!(fs::read_to_string(outside.join("keep")).unwrap(), "intact");
    fs::remove_dir(&alias).unwrap();
    drop(store);

    let workspace_alias = root.path().join("workspace-alias");
    let status = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&workspace_alias)
        .arg(&workspace)
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        housekeeping::prune_workspace(&cache, &workspace_alias, true, false)
            .unwrap()
            .deleted_entries,
        0
    );
    fs::remove_dir(&workspace_alias).unwrap();

    let moved = root.path().join("moved-workspace");
    fs::rename(&workspace, &moved).unwrap();
    fs::create_dir(&workspace).unwrap();
    assert_eq!(
        housekeeping::prune_workspace(&cache, &workspace, true, false)
            .unwrap()
            .deleted_entries,
        0
    );
    assert!(context.exists());
    fs::remove_dir(&workspace).unwrap();
    assert_eq!(
        housekeeping::prune_workspace(&cache, &workspace, true, false)
            .unwrap()
            .deleted_entries,
        1
    );
    assert!(!context.exists());
}
