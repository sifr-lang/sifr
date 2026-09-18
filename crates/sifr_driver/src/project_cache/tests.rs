use super::*;
use sifr_frontend::{
    DiskSourceProvider, FrontendContext, FrontendInput, FrontendMode, SourcePath, SourceText,
};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{
    fs,
    process::{Child, Command},
    thread,
    time::Duration,
};

fn context() -> SemanticInputs {
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
fn compute(file: &Path, provider: &mut dyn SourceProvider) -> Vec<RenderedDiagnostic> {
    let source = provider.read_file(file).unwrap();
    let mut frontend = FrontendContext::load_single_file(FrontendInput {
        path: SourcePath::new(file),
        source,
        mode: FrontendMode::SingleFile,
    })
    .unwrap();
    frontend.diagnostics_for_project().into_value().diagnostics
}
fn run(cache: &Path, file: &Path) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    check(
        cache,
        file,
        &mut DiskSourceProvider::new(),
        context(),
        &AtomicBool::new(false),
        |provider| compute(file, provider),
    )
}
fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let workspace = root.path().join("workspace");
    fs::create_dir(&workspace).unwrap();
    let file = workspace.join("main.sifr");
    fs::write(&file, "def main() -> None:\n    value: int = 1\n").unwrap();
    let cache = root.path().join("cache");
    (root, file, cache)
}
fn store(cache: &Path, file: &Path) -> storage::Store {
    storage::Store::open(
        cache,
        file.parent().unwrap(),
        &context().identity().unwrap(),
    )
    .unwrap()
}
#[test]
fn dx13_p01_p02_p08_p12_completed_families() {
    let (_root, file, cache) = fixture();
    let (fresh, first) = run(&cache, &file);
    assert_eq!(first.status, "published");
    let (restored, second) = run(&cache, &file);
    assert_eq!(fresh, restored);
    assert_eq!(second.restored_checks, 1);
    let generation = store(&cache, &file).latest().unwrap().unwrap();
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
    let generation = store(&cache, &file).latest().unwrap().unwrap();
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
#[test]
fn dx13_c08_c09_inherited_payloads_reader_gc() {
    let (_root, file, cache) = fixture();
    run(&cache, &file);
    let store = store(&cache, &file);
    let old = store.latest().unwrap().unwrap();
    let old_record = old.manifest.records.iter().next().unwrap().clone();
    let before = fs::read(old.path.join(&old_record)).unwrap();
    let other = file.parent().unwrap().join("other.sifr");
    fs::write(&other, "def main() -> None:\n    missing()\n").unwrap();
    let (_, published) = run(&cache, &other);
    assert_eq!(published.status, "published");
    let latest = store.latest().unwrap().unwrap();
    assert_eq!(latest.manifest.records.len(), 2);
    assert_eq!(
        fs::metadata(old.path.join(&old_record)).unwrap().ino(),
        fs::metadata(latest.path.join(&old_record)).unwrap().ino()
    );
    assert_eq!(fs::read(old.path.join(&old_record)).unwrap(), before);
    assert_eq!(store.prune(true, false).unwrap(), 0);
    assert!(old.path.exists());
    let mut unrelated = Command::new("sleep").arg("10").spawn().unwrap();
    drop(old);
    assert_eq!(store.prune(false, false).unwrap(), 0);
    assert_eq!(store.prune(true, true).unwrap(), 1);
    let removed = store.prune(true, false).unwrap();
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
    let generation = store.latest().unwrap().unwrap();
    let record = generation.manifest.records.iter().next().unwrap();
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
fn super_record(file: &Path) -> CompletedCheck {
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    let result = compute(file, &mut capture);
    CompletedCheck::capture(file, context(), &capture, &result).unwrap()
}
#[test]
fn dx13_p09_cancel_transient_and_changed_input() {
    let (_root, file, cache) = fixture();
    let cancel = AtomicBool::new(false);
    let (_, report) = check(
        &cache,
        &file,
        &mut DiskSourceProvider::new(),
        context(),
        &cancel,
        |provider| {
            let result = compute(&file, provider);
            cancel.store(true, Ordering::Release);
            result
        },
    );
    assert_eq!(report.status, "cancelled");
    assert!(store(&cache, &file).latest().unwrap().is_none());
    let (_, report) = check(
        &cache,
        &file,
        &mut DiskSourceProvider::new(),
        context(),
        &AtomicBool::new(false),
        |provider| {
            let result = compute(&file, provider);
            fs::write(&file, "def main() -> None:\n    pass\n").unwrap();
            result
        },
    );
    assert_eq!(report.status, "changed-inputs");
    let (_, report) = check(
        &cache,
        &file,
        &mut DiskSourceProvider::new(),
        context(),
        &AtomicBool::new(false),
        |provider| {
            provider.read_file(&file).unwrap();
            vec![crate::diagnostics::diagnostic_with_code(
                "temporary environment failure",
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        },
    );
    assert_eq!(report.status, "uncacheable");
    assert!(store(&cache, &file).latest().unwrap().is_none());
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
fn worker(file: &Path, cache: &Path, mode: &str, marker: &Path) -> Child {
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
        let generation = store.latest().unwrap().unwrap();
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
fn dx13_c09_p01_p09_process_death_and_new_process_restore() {
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
        assert!(store(&cache, &file).latest().unwrap().is_none());
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
    assert!(store(&cache, &file).latest().unwrap().is_none());
    assert_eq!(run(&cache, &file).1.status, "published");
}
#[test]
fn dx13_c09_concurrent_process_reader_and_gc() {
    let (root, file, cache) = fixture();
    run(&cache, &file);
    let store = store(&cache, &file);
    let old = store.latest().unwrap().unwrap().path.clone();
    let marker = root.path().join("reader");
    let mut reader = worker(&file, &cache, "reader", &marker);
    let deadline = Instant::now() + Duration::from_secs(15);
    while !marker.exists() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(marker.exists());
    fs::write(&file, "def main() -> None:\n    absent()\n").unwrap();
    assert_eq!(run(&cache, &file).1.status, "published");
    assert_eq!(store.prune(true, false).unwrap(), 0);
    assert!(old.exists());
    reader.kill().unwrap();
    reader.wait().unwrap();
    assert_eq!(store.prune(true, false).unwrap(), 1);
    assert!(!old.exists());
    assert_eq!(run(&cache, &file).1.restored_checks, 1);
}
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
