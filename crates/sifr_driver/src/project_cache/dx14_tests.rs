use super::*;
use sifr_frontend::{DiskSourceProvider, FrontendContext, ProjectRoot, SourcePath, SourceText};
use std::fs;

fn compiler() -> sifr_identity::CompilerIdentity {
    sifr_identity::CompilerIdentity::for_test(crate::compiled_input_tokens(), "dx14")
}
pub(super) fn inputs() -> SemanticInputs {
    SemanticInputs {
        compiler: compiler().as_str().into(),
        metadata: "metadata-generation".into(),
        target: "test-target".into(),
        workspace_and_source_policy: "captured-source-policy".into(),
        package_and_lock: "manifestless-owner-v1".into(),
        language_options: "test-options".into(),
        diagnostic_policy: "canonical".into(),
        components: BTreeMap::new(),
        required_external: Default::default(),
        external: BTreeMap::new(),
    }
}
fn frontend(file: &Path, provider: &mut dyn SourceProvider) -> FrontendContext {
    let mut capture = CapturingSourceProvider::new(provider);
    FrontendContext::load_project(
        &ProjectRoot {
            root: SourcePath::new(file.parent().unwrap()),
            entrypoint: SourcePath::new(file),
        },
        &mut capture,
    )
    .unwrap()
    .with_compiler_identity(compiler())
}
pub(super) fn run(
    cache: &Path,
    file: &Path,
    context: SemanticInputs,
) -> (Vec<RenderedDiagnostic>, ProjectCacheReport) {
    check(
        (cache, file.parent().unwrap()),
        file,
        &mut DiskSourceProvider::new(),
        context,
        &AtomicBool::new(false),
        Some((&compiler(), &Default::default(), None)),
        |provider| {
            let config = file.parent().unwrap().join("sifr.toml");
            if provider.is_file(&config) {
                provider.read_file(&config).unwrap();
            }
            frontend(file, provider)
                .diagnostics_for_project()
                .into_value()
                .diagnostics
                .into()
        },
    )
}
pub(super) fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let project = root.path().join("project");
    fs::create_dir(&project).unwrap();
    fs::write(project.join("sifr.toml"), "[source]\nroot = \".\"\n").unwrap();
    let file = project.join("main.sifr");
    fs::write(
        &file,
        "from helper import value

def main() -> int:
    return value()
",
    )
    .unwrap();
    fs::write(
        project.join("helper.sifr"),
        "def value() -> int:
    return 1
",
    )
    .unwrap();
    let cache = root.path().join("cache");
    (root, file, cache)
}

#[test]
fn disk_and_memory_reuse_changed_ordinary_body() {
    let (_root, file, cache) = fixture();
    assert_eq!(run(&cache, &file, inputs()).1.status, "published");
    let mut memory = frontend(&file, &mut DiskSourceProvider::new());
    assert!(
        memory
            .diagnostics_for_project()
            .value()
            .diagnostics
            .is_empty()
    );
    let graph = memory.module_graph();
    let helper = graph
        .modules
        .iter()
        .find(|m| m.canonical_path.as_path().ends_with("helper.sifr"))
        .unwrap()
        .id;
    let main = graph.entrypoint;
    let changed = "def value() -> int:
    return 2
";
    fs::write(file.parent().unwrap().join("helper.sifr"), changed).unwrap();
    let report = memory
        .update_module_source(helper, SourceText::new(changed), None)
        .unwrap();
    assert_eq!(report.invalidated_modules, vec![helper]);
    assert_eq!(
        memory.diagnostics_for_module(main).metadata().cache_status,
        sifr_frontend::CacheStatus::Hit
    );
    let (diagnostics, report) = run(&cache, &file, inputs());
    assert!(diagnostics.is_empty());
    assert_eq!(report.status, "interface-restored", "{report:?}");
    assert!(
        report
            .modules
            .iter()
            .any(|m| m.path == file && m.action == "restored")
    );
    assert!(
        report
            .modules
            .iter()
            .any(|m| m.path.ends_with("helper.sifr") && m.action == "computed")
    );
    let fresh = frontend(&file, &mut DiskSourceProvider::new())
        .diagnostics_for_project()
        .into_value()
        .diagnostics;
    assert_eq!(diagnostics, fresh);
}

#[test]
fn body_errors_defaults_constants_and_generics_are_conservative() {
    let cases = [
        (
            "@const_eval\ndef constant() -> int:\n    return 1\ndef value() -> int:\n    return 1\n",
            "@const_eval\ndef constant() -> int:\n    return 2\ndef value() -> int:\n    return 1\n",
            false,
        ),
        (
            "def value() -> int:
    return 1
",
            "def value() -> int:
    return \"bad\"
",
            true,
        ),
        (
            "def value(offset: int = 1) -> int:
    return offset
",
            "def value(offset: int = 2) -> int:
    return offset
",
            false,
        ),
        (
            "NUMBER: int = 1
def value() -> int:
    return NUMBER
",
            "NUMBER: int = 2
def value() -> int:
    return NUMBER
",
            false,
        ),
        (
            "def identity[T](x: T) -> T:
    return x
def value() -> int:
    return 1
",
            "def identity[T](x: T) -> T:
    y: T = x
    return y
def value() -> int:
    return 1
",
            false,
        ),
    ];
    for (before, after, error) in cases {
        let (_root, file, cache) = fixture();
        let helper_path = file.parent().unwrap().join("helper.sifr");
        fs::write(&helper_path, before).unwrap();
        assert!(run(&cache, &file, inputs()).0.is_empty());
        let mut memory = frontend(&file, &mut DiskSourceProvider::new());
        assert!(
            memory
                .diagnostics_for_project()
                .value()
                .diagnostics
                .is_empty()
        );
        let graph = memory.module_graph();
        let helper = graph
            .modules
            .iter()
            .find(|m| m.canonical_path.as_path() == helper_path)
            .unwrap()
            .id;
        fs::write(&helper_path, after).unwrap();
        let invalidation = memory
            .update_module_source(helper, SourceText::new(after), None)
            .unwrap();
        assert!(
            invalidation.invalidated_modules.contains(&graph.entrypoint),
            "{after}"
        );
        let (actual, report) = run(&cache, &file, inputs());
        assert_ne!(report.status, "interface-restored");
        assert_eq!(!actual.is_empty(), error, "{after}: {actual:?}");
        let fresh = frontend(&file, &mut DiskSourceProvider::new())
            .diagnostics_for_project()
            .into_value()
            .diagnostics;
        assert_eq!(actual, fresh);
        assert_eq!(
            memory.diagnostics_for_project().into_value().diagnostics,
            fresh
        );
    }
}

#[test]
fn context_changes_never_restore_live_authority() {
    let (_root, file, cache) = fixture();
    assert!(run(&cache, &file, inputs()).0.is_empty());
    for kind in ["sql", "python", "component"] {
        let mut context = inputs();
        context
            .components
            .insert(kind.into(), "changed-authority".into());
        context.required_external.insert(kind.into());
        context
            .external
            .insert(kind.into(), "declared-context-v2".into());
        let (_, report) = run(&cache, &file, context);
        assert_eq!(report.restored_checks, 0);
        assert!(
            report
                .modules
                .iter()
                .all(|module| module.action == "computed")
        );
    }
}

#[test]
fn dx14_bounded_edits_match_fresh_with_independent_error_expectations() {
    let (_root, file, cache) = fixture();
    let mut seed = 0x5eed_u64;
    let mut reused = 0;
    for step in 0..24 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let bad = step % 5 == 3;
        let source = if bad {
            "def value() -> int:
    return \"bad\"
"
            .to_owned()
        } else {
            format!(
                "def value() -> int:
    return {}
",
                seed % 10000
            )
        };
        fs::write(file.parent().unwrap().join("helper.sifr"), source).unwrap();
        let (actual, report) = run(&cache, &file, inputs());
        reused += usize::from(report.status == "interface-restored");
        let fresh = frontend(&file, &mut DiskSourceProvider::new())
            .diagnostics_for_project()
            .into_value()
            .diagnostics;
        assert_eq!(actual, fresh, "step {step}");
        assert_eq!(!actual.is_empty(), bad, "step {step}");
    }
    assert!(reused >= 10, "caching disabled is not acceptance: {reused}");
}

#[test]
fn dx14_editor_saved_overlay_stale_and_deeper_family_boundaries() {
    let (_root, file, cache) = fixture();
    assert_eq!(run(&cache, &file, inputs()).1.status, "published");
    let store = storage::Store::open(
        &cache,
        file.parent().unwrap(),
        &inputs().identity().unwrap(),
    )
    .unwrap();
    let generation = store.latest().unwrap();
    let record = generation.records().next().unwrap().unwrap();
    let mut saved = frontend(&file, &mut DiskSourceProvider::new());
    let entry = saved.module_graph().entrypoint;
    let decisions = saved
        .restore_completed_checks(&record, &inputs(), &mut DiskSourceProvider::new(), false)
        .unwrap();
    assert!(
        decisions
            .iter()
            .all(|decision| decision.action == "restored")
    );
    assert_eq!(
        saved.diagnostics_for_module(entry).metadata().cache_status,
        sifr_frontend::CacheStatus::Hit
    );
    assert_eq!(
        saved.lower_module(entry).metadata().cache_status,
        sifr_frontend::CacheStatus::Miss,
        "restored diagnostics do not claim a typed family"
    );

    let mut overlay = frontend(&file, &mut DiskSourceProvider::new());
    overlay
        .update_module_source(
            entry,
            SourceText::new("def main() -> int:\n    return \"bad\"\n"),
            None,
        )
        .unwrap();
    assert!(
        overlay
            .restore_completed_checks(&record, &inputs(), &mut DiskSourceProvider::new(), false)
            .is_none()
    );
    assert!(
        !overlay
            .diagnostics_for_module(entry)
            .value()
            .diagnostics
            .is_empty()
    );

    let mut old_snapshot = frontend(&file, &mut DiskSourceProvider::new());
    fs::write(&file, "def main() -> int:\n    return \"bad\"\n").unwrap();
    assert!(
        old_snapshot
            .restore_completed_checks(&record, &inputs(), &mut DiskSourceProvider::new(), false)
            .is_none(),
        "disk changes cannot relabel a captured old generation"
    );
    let mut current = frontend(&file, &mut DiskSourceProvider::new());
    assert!(
        current
            .restore_completed_checks(&record, &inputs(), &mut DiskSourceProvider::new(), false)
            .is_none()
    );
    assert!(
        !current
            .diagnostics_for_module(entry)
            .value()
            .diagnostics
            .is_empty()
    );
    let (_, report) = run(&cache, &file, inputs());
    assert_eq!(report.restored_checks, 0);
}

#[test]
fn dx14_reconfiguration_deletion_and_unknown_effect_scope() {
    let (_root, file, cache) = fixture();
    assert!(run(&cache, &file, inputs()).0.is_empty());
    let store = storage::Store::open(
        &cache,
        file.parent().unwrap(),
        &inputs().identity().unwrap(),
    )
    .unwrap();
    let generation = store.latest().unwrap();
    let record = generation.records().next().unwrap().unwrap();
    fs::remove_file(file.parent().unwrap().join("helper.sifr")).unwrap();
    let mut disk = DiskSourceProvider::new();
    let mut capture = CapturingSourceProvider::new(&mut disk);
    assert!(
        interface_reuse::restore(
            &record,
            &file,
            &inputs(),
            &mut capture,
            &compiler(),
            Default::default(),
            None,
        )
        .is_none()
    );
    for (before, after) in [
        (
            "def value() -> int:\n    return 1\n",
            "def value() -> int:\n    print(1)\n    return 1\n",
        ),
        (
            "def consume(value: str) -> str:\n    return value\n",
            "def consume(own value: str) -> str:\n    return value\n",
        ),
        (
            "async def value() -> int:\n    return 1\n",
            "async def value() -> int:\n    return 2\n",
        ),
    ] {
        let helper = file.parent().unwrap().join("helper.sifr");
        // Keep the imported callable stable; a second changed declaration still
        // invalidates consumers conservatively for unknown effects/ownership.
        let prefix = if before.contains("consume") || before.starts_with("async") {
            "def value() -> int:\n    return 1\n"
        } else {
            ""
        };
        let before = if before.starts_with("async") {
            before.replace("value", "worker")
        } else {
            before.into()
        };
        let after = if after.starts_with("async") {
            after.replace("value", "worker")
        } else {
            after.into()
        };
        fs::write(&helper, format!("{prefix}{before}")).unwrap();
        let mut memory = frontend(&file, &mut DiskSourceProvider::new());
        let graph = memory.module_graph();
        let helper_id = graph
            .modules
            .iter()
            .find(|m| m.canonical_path.as_path() == helper)
            .unwrap()
            .id;
        let update = memory
            .update_module_source(helper_id, SourceText::new(format!("{prefix}{after}")), None)
            .unwrap();
        assert!(update.invalidated_modules.contains(&graph.entrypoint));
    }
}

#[test]
fn dx14_restored_interface_reports_unavailable_publication() {
    use std::os::unix::fs::PermissionsExt;
    let (_root, file, cache) = fixture();
    assert_eq!(run(&cache, &file, inputs()).1.status, "published");
    let store = storage::Store::open(
        &cache,
        file.parent().unwrap(),
        &inputs().identity().unwrap(),
    )
    .unwrap();
    fs::set_permissions(&store.root, fs::Permissions::from_mode(0o500)).unwrap();
    fs::write(
        file.parent().unwrap().join("helper.sifr"),
        "def value() -> int:\n    return 7\n",
    )
    .unwrap();
    let (diagnostics, report) = run(&cache, &file, inputs());
    fs::set_permissions(&store.root, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(diagnostics.is_empty());
    assert_eq!(report.status, "interface-restored-write-unavailable");
    assert_eq!(report.restored_checks, 1);
    assert!(
        report
            .modules
            .iter()
            .any(|module| module.action == "restored" && module.path == file)
    );
}

#[test]
fn dx15_workspace_owned_saved_validation_rejects_changed_disk_and_overlay() {
    let (_root, file, cache) = fixture();
    assert_eq!(run(&cache, &file, inputs()).1.status, "published");
    let store = storage::Store::open(
        &cache,
        file.parent().unwrap(),
        &inputs().identity().unwrap(),
    )
    .unwrap();
    let generation = store.latest().unwrap();
    let record = generation.records().next().unwrap().unwrap();
    let open = || {
        sifr_frontend::WorkspaceSession::open_project(ProjectRoot {
            root: SourcePath::new(file.parent().unwrap()),
            entrypoint: SourcePath::new(&file),
        })
        .unwrap()
        .with_compiler_identity(compiler())
    };
    let mut saved = open();
    assert!(
        saved
            .restore_saved_checks(&record, &inputs())
            .unwrap()
            .iter()
            .all(|decision| decision.action == "restored")
    );
    let mut overlay = open();
    let frontend = overlay.context_mut().unwrap();
    let entry = frontend.module_graph().entrypoint;
    frontend
        .update_module_source(
            entry,
            SourceText::new("def main() -> int:\n    return \"bad\"\n"),
            None,
        )
        .unwrap();
    assert!(overlay.restore_saved_checks(&record, &inputs()).is_none());
    let mut old_snapshot = open();
    fs::write(&file, "def main() -> int:\n    return \"bad\"\n").unwrap();
    assert!(
        old_snapshot
            .restore_saved_checks(&record, &inputs())
            .is_none(),
        "workspace ownership must still reobserve disk, not just validate the captured snapshot"
    );
}
