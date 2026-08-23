use crate::{
    CacheStatus, DiskSourceProvider, DocumentVersion, FrontendContext, FrontendInput, FrontendMode,
    ModuleId, OverlayDocument, OverlaySourceProvider, ProjectRoot, SourceDependencyKind,
    SourcePath, SourceText, TrackingSourceProvider, WorkspaceDirtyReason, WorkspaceDirtyScope,
};

fn input(source: &str) -> FrontendInput {
    FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    }
}

#[test]
fn single_file_queries_are_cached_and_deterministic() {
    let mut context = FrontendContext::load_single_file(input(
        "def main():\n    value: int = 1\n    reveal_type(value)\n",
    ))
    .expect("context should load");

    let main_module = ModuleId(0);
    let first = context.diagnostics_for_module(main_module);
    let second = context.diagnostics_for_module(main_module);

    assert_eq!(first.value().module, main_module);
    assert_eq!(second.metadata().cache_status, CacheStatus::Hit);
    assert_eq!(first.value().diagnostics, second.value().diagnostics);
}

#[test]
fn source_update_invalidates_cached_queries() {
    let mut context = FrontendContext::load_single_file(input("def main():\n    return 1\n"))
        .expect("context should load");
    let main_module = ModuleId(0);
    let _ = context.diagnostics_for_module(main_module);

    let report = context
        .update_module_source(
            main_module,
            SourceText::new("def main():\n    return 2\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("update should succeed");

    assert!(report.invalidated_modules.contains(&main_module));
    assert_eq!(
        context
            .diagnostics_for_module(main_module)
            .metadata()
            .cache_status,
        CacheStatus::Miss
    );
}

#[test]
fn private_module_body_update_stays_local_when_signatures_match() {
    let dir = temp_project_dir("private_body_invalidation");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main() -> int:\n    return value()\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(1);
    let main = ModuleId(0);
    assert!(
        context
            .diagnostics_for_project()
            .into_value()
            .diagnostics
            .is_empty()
    );

    let report = context
        .update_module_source(
            helper,
            SourceText::new("def value() -> int:\n    return 2\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![helper]);
    assert!(!report.invalidated_modules.contains(&main));
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::OneModule {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![WorkspaceDirtyReason::SourceTextChanged]
    );
}

#[test]
fn private_body_update_with_unchanged_imports_stays_local() {
    let dir = temp_project_dir("private_body_import_signature");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main() -> int:\n    return value()\n",
    )
    .expect("main should be written");
    std::fs::write(dir.join("dep.sifr"), "other: int = 1\n").expect("dep should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "from dep import other\n\ndef value() -> int:\n    return other\n",
    )
    .expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(2);
    let main = ModuleId(0);
    assert!(
        context
            .diagnostics_for_project()
            .into_value()
            .diagnostics
            .is_empty()
    );

    let report = context
        .update_module_source(
            helper,
            SourceText::new("from dep import other\n\ndef value() -> int:\n    return other + 1\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![helper]);
    assert!(!report.invalidated_modules.contains(&main));
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::OneModule {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![WorkspaceDirtyReason::SourceTextChanged]
    );
}

#[test]
fn public_export_update_invalidates_reverse_dependents() {
    let dir = temp_project_dir("export_signature_invalidation");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main() -> int:\n    return value()\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(1);
    let main = ModuleId(0);
    let _ = context.diagnostics_for_project();

    let report = context
        .update_module_source(
            helper,
            SourceText::new("def value() -> str:\n    return \"changed\"\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![main, helper]);
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::ReverseDependencies {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::ExportSignatureChanged
        ]
    );
}

#[test]
fn import_signature_update_selects_graph_scope() {
    let dir = temp_project_dir("import_signature_invalidation");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main() -> int:\n    return value()\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("other.sifr"),
        "def other() -> int:\n    return 2\n",
    )
    .expect("other should be written");
    let mut context = load_temp_project(&dir);
    let main = ModuleId(0);
    let _ = context.diagnostics_for_project();

    let report = context
        .update_module_source(
            main,
            SourceText::new("from other import other\n\ndef main() -> int:\n    return other()\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("main update should succeed");

    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::GraphStructure
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::ImportSignatureChanged
        ]
    );
}

#[test]
fn project_graph_records_local_import_edges() {
    let dir = std::env::temp_dir().join(format!(
        "sifr_frontend_project_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value)\n",
    )
    .expect("main should be written");
    std::fs::write(dir.join("helper.sifr"), "value: int = 1\n").expect("helper should be written");

    let mut provider = DiskSourceProvider::new();
    let mut context = FrontendContext::load_project(
        &ProjectRoot {
            root: SourcePath::new(&dir),
            entrypoint: SourcePath::new(dir.join("main.sifr")),
        },
        &mut provider,
    )
    .expect("project should load");

    let graph = context.module_graph();
    assert_eq!(graph.entrypoint, ModuleId(0));
    assert_eq!(graph.edges.len(), 1);

    let diagnostics = context.diagnostics_for_project().into_value().diagnostics;
    assert!(
        diagnostics.is_empty(),
        "project diagnostics should consume dependency exports from the canonical frontend: {diagnostics:?}"
    );
}

#[test]
fn project_loading_uses_overlay_and_tracking_provider() {
    let dir = std::env::temp_dir().join(format!(
        "sifr_frontend_project_overlay_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    let main_path = dir.join("main.sifr");
    let helper_path = dir.join("helper.sifr");
    std::fs::write(
        &main_path,
        "from helper import value\n\ndef main():\n    print(value)\n",
    )
    .expect("main should be written");
    std::fs::write(&helper_path, "value: int = 1\n").expect("helper should be written");

    let mut overlay = OverlaySourceProvider::new(DiskSourceProvider::new());
    overlay.insert_overlay(OverlayDocument::new(
        SourcePath::new(&helper_path),
        None,
        DocumentVersion::new(5),
        SourceText::new("value: int = 2\n"),
        Some("value: int = 1\n"),
    ));
    let mut provider = TrackingSourceProvider::new(overlay);

    let context = FrontendContext::load_project(
        &ProjectRoot {
            root: SourcePath::new(&dir),
            entrypoint: SourcePath::new(&main_path),
        },
        &mut provider,
    )
    .expect("project should load through provider");

    assert!(
        provider
            .dependencies()
            .iter()
            .any(|dependency| dependency.kind == SourceDependencyKind::DirectoryRead)
    );
    assert!(context.source_map().files.iter().any(|file| {
        file.canonical_path.as_path() == helper_path && file.source.as_str() == "value: int = 2\n"
    }));
}

fn temp_project_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sifr_frontend_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    dir
}

fn load_temp_project(dir: &std::path::Path) -> FrontendContext {
    let mut provider = DiskSourceProvider::new();
    FrontendContext::load_project(
        &ProjectRoot {
            root: SourcePath::new(dir),
            entrypoint: SourcePath::new(dir.join("main.sifr")),
        },
        &mut provider,
    )
    .expect("project should load")
}
