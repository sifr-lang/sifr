use super::{
    WorkspaceDirtyReason, WorkspaceDirtyScope, WorkspaceDirtyScopeReport, WorkspaceSession,
    WorkspaceSessionTarget,
};
use crate::{
    CompilerFingerprint, DocumentVersion, FrontendMode, ProjectResidencyKind, ProjectRoot,
    SifrBuildInfoCandidate, SifrBuildInfoRejection, SifrBuildInfoSource, SifrBuildInfoVerification,
    SourceDependencyKind, SourceHash, SourcePath, SourceText, WatchRegistrationReason,
    WorkspaceTracePhase,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn project_session_snapshot_records_overlay_and_dependencies() {
    let temp = TempProject::new("workspace_session_project");
    temp.write("main.sifr", "from helper import value\nprint(value)\n");
    temp.write("helper.sifr", "value = 1\n");
    let root = ProjectRoot {
        root: SourcePath::new(temp.root.clone()),
        entrypoint: SourcePath::new(temp.root.join("main.sifr")),
    };
    let expected_root = root.root.clone();
    let expected_entrypoint = root.entrypoint.clone();
    let mut session = WorkspaceSession::open_project(root).expect("project opens");
    let opened_revision = session.revision().as_u64();
    assert_eq!(
        session.snapshot().dirty_scope_report.scope,
        WorkspaceDirtyScope::None
    );

    session.upsert_overlay(
        SourcePath::new(temp.root.join("helper.sifr")),
        Some("file:///helper.sifr".to_string()),
        DocumentVersion::new(4),
        SourceText::new("value = 2\n"),
        Some("value = 1\n"),
    );
    assert_eq!(session.revision().as_u64(), opened_revision + 1);
    session.reload().expect("overlay-backed reload succeeds");
    assert_eq!(session.revision().as_u64(), opened_revision + 2);
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.dirty_scope_report.scope,
        WorkspaceDirtyScope::OneModule {
            path: SourcePath::new(temp.root.join("helper.sifr"))
        }
    );
    assert_eq!(
        snapshot.dirty_scope_report.reasons,
        vec![WorkspaceDirtyReason::SourceTextChanged]
    );

    assert!(matches!(
        snapshot.target,
        WorkspaceSessionTarget::Project(_)
    ));
    assert_eq!(snapshot.revision, session.revision());
    assert_eq!(
        snapshot.compiler_options.mode,
        FrontendMode::ProjectEntrypoint
    );
    assert_eq!(
        snapshot.package_config_identity.workspace_root,
        Some(expected_root)
    );
    assert_eq!(
        snapshot.package_config_identity.entrypoint,
        Some(expected_entrypoint)
    );
    assert_eq!(snapshot.cache_registry.parse_generation, 0);
    assert_eq!(snapshot.cache_registry.hir_generation, 0);
    assert_eq!(snapshot.overlays.len(), 1);
    assert!(!snapshot.overlays[0].matches_disk);
    assert_eq!(snapshot.overlays[0].source.as_str(), "value = 2\n");
    assert!(snapshot
        .source_dependencies
        .iter()
        .any(|dependency| matches!(dependency.kind, SourceDependencyKind::FileRead)));
    assert!(snapshot
        .residency
        .projects
        .iter()
        .any(|project| project.kind == ProjectResidencyKind::ExplicitApiOpen && project.loaded));
    assert!(snapshot.residency.configs.iter().any(|config| config
        .path
        .as_path()
        .ends_with("sifr.toml")
        && !config.pending_reload));
    assert!(snapshot.residency.watchers.iter().any(|watcher| watcher
        .reasons
        .contains(&WatchRegistrationReason::PackageRoot)));
    assert!(snapshot
        .residency
        .watchers
        .iter()
        .any(|watcher| watcher.reasons.contains(&WatchRegistrationReason::SeenFile)));
    assert_eq!(
        snapshot
            .source_map
            .as_ref()
            .expect("source map exists")
            .files
            .len(),
        2
    );
    assert_eq!(
        snapshot
            .module_graph
            .as_ref()
            .expect("module graph exists")
            .modules
            .len(),
        2
    );

    let removed = session.remove_overlay(&temp.root.join("helper.sifr"));
    assert!(removed.is_some());
    assert_eq!(session.revision().as_u64(), snapshot.revision.as_u64() + 1);
    let removed_snapshot = session.snapshot();
    assert!(removed_snapshot.overlays.is_empty());
    assert!(!removed_snapshot
        .residency
        .projects
        .iter()
        .any(
            |project| project.kind == ProjectResidencyKind::OpenFileOwner
                && project
                    .root
                    .as_ref()
                    .is_some_and(|path| path.as_path().ends_with("helper.sifr"))
        ));
    assert_eq!(
        removed_snapshot.dirty_scope_report.reasons,
        vec![WorkspaceDirtyReason::SourceTextChanged]
    );
    assert!(removed_snapshot
        .residency
        .watchers
        .iter()
        .any(|watcher| watcher.reasons.contains(&WatchRegistrationReason::SeenFile)));
}

#[test]
fn debug_snapshot_explains_invalidation_and_status_counts() {
    let temp = TempProject::new("workspace_session_debug");
    temp.write("main.sifr", "from helper import value\nprint(value)\n");
    temp.write("helper.sifr", "value = 1\n");
    let mut session = WorkspaceSession::open_project(ProjectRoot {
        root: SourcePath::new(temp.root.clone()),
        entrypoint: SourcePath::new(temp.root.join("main.sifr")),
    })
    .expect("project opens");

    session.upsert_overlay(
        SourcePath::new(temp.root.join("helper.sifr")),
        Some("file:///helper.sifr".to_string()),
        DocumentVersion::new(2),
        SourceText::new("value = 2\n"),
        Some("value = 1\n"),
    );
    session.reload().expect("reload after dependency edit");
    let debug = session.snapshot().debug;

    assert_eq!(debug.status.target_kind, "project");
    assert_eq!(debug.status.open_file_count, 1);
    assert_eq!(debug.status.source_file_count, 2);
    assert_eq!(debug.status.module_count, 2);
    assert!(debug.status.memory.source_text_bytes > 0);
    assert!(debug.status.memory.retained_watchers > 0);
    assert!(debug
        .trace
        .events
        .iter()
        .any(|event| event.phase == WorkspaceTracePhase::SourceUpdate
            && event.detail.contains("overlay_upsert")));
    assert!(debug
        .trace
        .events
        .iter()
        .any(|event| event.phase == WorkspaceTracePhase::Invalidation
            && event.detail.contains("SourceTextChanged")));
    let rendered = debug.render_text();
    assert!(rendered.contains("[status]"));
    assert!(rendered.contains("phase=parse"));
}

#[test]
fn closing_one_overlay_preserves_other_overlay_watcher() {
    let temp = TempProject::new("workspace_session_close_watchers");
    temp.write(
        "main.sifr",
        "from first import a\nfrom second import b\nprint(a + b)\n",
    );
    temp.write("first.sifr", "a = 1\n");
    temp.write("second.sifr", "b = 2\n");
    let root = ProjectRoot {
        root: SourcePath::new(temp.root.clone()),
        entrypoint: SourcePath::new(temp.root.join("main.sifr")),
    };
    let mut session = WorkspaceSession::open_project(root).expect("project opens");
    session.upsert_overlay(
        SourcePath::new(temp.root.join("first.sifr")),
        Some("file:///first.sifr".to_string()),
        DocumentVersion::new(2),
        SourceText::new("a = 3\n"),
        Some("a = 1\n"),
    );
    session.upsert_overlay(
        SourcePath::new(temp.root.join("second.sifr")),
        Some("file:///second.sifr".to_string()),
        DocumentVersion::new(2),
        SourceText::new("b = 4\n"),
        Some("b = 2\n"),
    );
    session.reload().expect("overlay-backed reload succeeds");

    assert!(session
        .remove_overlay(&temp.root.join("first.sifr"))
        .is_some());
    let snapshot = session.snapshot();
    let watcher_globs = snapshot
        .residency
        .watchers
        .iter()
        .map(|watcher| watcher.glob.clone())
        .collect::<BTreeSet<_>>();

    assert!(watcher_globs
        .iter()
        .any(|glob| glob.ends_with("second.sifr")));
    assert!(!snapshot
        .residency
        .projects
        .iter()
        .any(
            |project| project.kind == ProjectResidencyKind::OpenFileOwner
                && project
                    .root
                    .as_ref()
                    .is_some_and(|path| path.as_path().ends_with("first.sifr"))
        ));
    assert!(snapshot
        .residency
        .projects
        .iter()
        .any(
            |project| project.kind == ProjectResidencyKind::OpenFileOwner
                && project
                    .root
                    .as_ref()
                    .is_some_and(|path| path.as_path().ends_with("second.sifr"))
        ));
    assert_eq!(
        watcher_globs.len(),
        snapshot.residency.watchers.len(),
        "watch registrations should remain deduped by glob"
    );
}

#[test]
fn config_registry_pending_reload_and_extra_watch_roots_are_snapshot_visible() {
    let temp = TempProject::new("workspace_session_registry");
    temp.write("main.sifr", "print(1)\n");
    let root = ProjectRoot {
        root: SourcePath::new(temp.root.clone()),
        entrypoint: SourcePath::new(temp.root.join("main.sifr")),
    };
    let mut session = WorkspaceSession::open_project(root).expect("project opens");
    session.retain_stdlib_root(SourcePath::new(temp.root.join("stdlib")));
    session.retain_generated_artifact(SourcePath::new(temp.root.join(".sifrbuildinfo")));
    session.mark_config_pending_reload(SourcePath::new(temp.root.join("sifr.toml")));

    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.dirty_scope_report.scope,
        WorkspaceDirtyScope::ConfigProject
    );
    assert!(snapshot
        .residency
        .configs
        .iter()
        .any(|config| { config.path.as_path().ends_with("sifr.toml") && config.pending_reload }));
    assert!(snapshot.residency.watchers.iter().any(|watcher| watcher
        .reasons
        .contains(&WatchRegistrationReason::StdlibRoot)));
    assert!(snapshot.residency.watchers.iter().any(|watcher| watcher
        .reasons
        .contains(&WatchRegistrationReason::GeneratedArtifact)));

    session
        .reload()
        .expect("unrelated reload should preserve pending config");
    let reloaded = session.snapshot();
    assert!(reloaded.residency.configs.iter().any(|config| config
        .path
        .as_path()
        .ends_with("sifr.toml")
        && config.pending_reload));
}

#[test]
fn build_info_is_verified_against_current_workspace_fingerprints() {
    let temp = TempProject::new("workspace_session_build_info");
    temp.write("main.sifr", "print(1)\n");
    let root = ProjectRoot {
        root: SourcePath::new(temp.root.clone()),
        entrypoint: SourcePath::new(temp.root.join("main.sifr")),
    };
    let mut session = WorkspaceSession::open_project(root).expect("project opens");
    let snapshot = session.snapshot();
    let source_map = snapshot.source_map.as_ref().expect("source map exists");
    let sources = source_map
        .files
        .iter()
        .map(|file| SifrBuildInfoSource {
            path: file.canonical_path.clone(),
            source_hash: file.source_hash.clone(),
        })
        .collect::<Vec<_>>();
    let candidate = SifrBuildInfoCandidate {
        path: SourcePath::new(temp.root.join(".sifrbuildinfo")),
        compiler: CompilerFingerprint::current(),
        package_config_identity: snapshot.package_config_identity.as_ref().clone(),
        sources,
    };

    let verification = session.verify_build_info(candidate.clone());
    assert!(matches!(
        verification,
        SifrBuildInfoVerification::Verified(_)
    ));
    assert!(session.snapshot().residency.build_info.is_some());

    let mut stale = candidate;
    stale.sources[0].source_hash = SourceHash::new("stale");
    let rejected = session.verify_build_info(stale);
    assert!(matches!(
        rejected,
        SifrBuildInfoVerification::Rejected { reasons, .. }
            if reasons.contains(&SifrBuildInfoRejection::SourceHashMismatch)
    ));
    assert!(session.snapshot().residency.build_info.is_none());
}

#[test]
fn build_info_rejects_missing_extra_package_and_unloaded_inputs() {
    let temp = TempProject::new("workspace_session_build_info_reject");
    temp.write("main.sifr", "print(1)\n");
    let root = ProjectRoot {
        root: SourcePath::new(temp.root.clone()),
        entrypoint: SourcePath::new(temp.root.join("main.sifr")),
    };
    let mut session = WorkspaceSession::open_project(root.clone()).expect("project opens");
    let snapshot = session.snapshot();
    let mut candidate = SifrBuildInfoCandidate {
        path: SourcePath::new(temp.root.join(".sifrbuildinfo")),
        compiler: CompilerFingerprint::current(),
        package_config_identity: snapshot.package_config_identity.as_ref().clone(),
        sources: Vec::new(),
    };

    let missing = session.verify_build_info(candidate.clone());
    assert!(matches!(
        missing,
        SifrBuildInfoVerification::Rejected { reasons, .. }
            if reasons.contains(&SifrBuildInfoRejection::MissingSource)
    ));

    candidate.sources = snapshot
        .source_map
        .as_ref()
        .expect("source map exists")
        .files
        .iter()
        .map(|file| SifrBuildInfoSource {
            path: file.canonical_path.clone(),
            source_hash: file.source_hash.clone(),
        })
        .collect();
    candidate.sources.push(SifrBuildInfoSource {
        path: SourcePath::new(temp.root.join("deleted.sifr")),
        source_hash: SourceHash::new("deleted"),
    });
    candidate.package_config_identity = Default::default();
    let rejected = session.verify_build_info(candidate);
    assert!(matches!(
        rejected,
        SifrBuildInfoVerification::Rejected { reasons, .. }
            if reasons.contains(&SifrBuildInfoRejection::ExtraSource)
                && reasons.contains(&SifrBuildInfoRejection::PackageConfigMismatch)
    ));

    let mut lazy = WorkspaceSession::project(root);
    let unloaded = lazy.verify_build_info(SifrBuildInfoCandidate {
        path: SourcePath::new(temp.root.join(".sifrbuildinfo")),
        compiler: CompilerFingerprint::current(),
        package_config_identity: Default::default(),
        sources: Vec::new(),
    });
    assert!(matches!(
        unloaded,
        SifrBuildInfoVerification::Rejected { reasons, .. }
            if reasons.contains(&SifrBuildInfoRejection::WorkspaceNotLoaded)
    ));
}

#[test]
fn dirty_scope_reports_merge_by_conservative_priority() {
    let temp = TempProject::new("workspace_session_dirty_scope");
    temp.write("main.sifr", "print(1)\n");
    temp.write("first.sifr", "print(1)\n");
    temp.write("second.sifr", "print(2)\n");
    let root = ProjectRoot {
        root: SourcePath::new(temp.root.clone()),
        entrypoint: SourcePath::new(temp.root.join("main.sifr")),
    };
    let mut session = WorkspaceSession::open_project(root).expect("project opens");

    session.upsert_overlay(
        SourcePath::new(temp.root.join("first.sifr")),
        Some("file:///first.sifr".to_string()),
        DocumentVersion::new(2),
        SourceText::new("print(3)\n"),
        Some("print(1)\n"),
    );
    session.record_dirty_scope(super::WorkspaceDirtyScopeReport::new(
        WorkspaceDirtyScope::OneModule {
            path: SourcePath::new(temp.root.join("second.sifr")),
        },
        vec![WorkspaceDirtyReason::FailedLookupChanged],
    ));
    let degraded_snapshot = session.snapshot();
    assert_eq!(
        degraded_snapshot.dirty_scope_report.scope,
        WorkspaceDirtyScope::GraphStructure
    );
    assert_eq!(
        degraded_snapshot.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::FailedLookupChanged
        ]
    );

    session.record_watcher_events(2, 64);
    let graph_snapshot = session.snapshot();
    assert_eq!(
        graph_snapshot.dirty_scope_report.scope,
        WorkspaceDirtyScope::GraphStructure
    );
    assert_eq!(
        graph_snapshot.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::FailedLookupChanged,
            WorkspaceDirtyReason::DirectoryEntriesChanged
        ]
    );

    session.record_watcher_events(65, 64);
    let storm_snapshot = session.snapshot();
    assert_eq!(
        storm_snapshot.dirty_scope_report.scope,
        WorkspaceDirtyScope::Workspace
    );
    assert_eq!(
        storm_snapshot.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::FailedLookupChanged,
            WorkspaceDirtyReason::DirectoryEntriesChanged,
            WorkspaceDirtyReason::WatcherStorm
        ]
    );
}

#[test]
fn dirty_scope_report_merge_covers_dependency_and_config_scopes() {
    let first = SourcePath::new("first.sifr");
    let second = SourcePath::new("second.sifr");
    let mut report = WorkspaceDirtyScopeReport::new(
        WorkspaceDirtyScope::ReverseDependencies {
            path: first.clone(),
        },
        vec![WorkspaceDirtyReason::ImportSignatureChanged],
    );

    report.merge(WorkspaceDirtyScopeReport::new(
        WorkspaceDirtyScope::ReverseDependencies { path: first },
        vec![WorkspaceDirtyReason::ExportSignatureChanged],
    ));
    assert_eq!(
        report.scope,
        WorkspaceDirtyScope::ReverseDependencies {
            path: SourcePath::new("first.sifr")
        }
    );
    assert_eq!(
        report.reasons,
        vec![
            WorkspaceDirtyReason::ImportSignatureChanged,
            WorkspaceDirtyReason::ExportSignatureChanged
        ]
    );

    report.merge(WorkspaceDirtyScopeReport::new(
        WorkspaceDirtyScope::ReverseDependencies { path: second },
        vec![WorkspaceDirtyReason::FailedLookupChanged],
    ));
    assert_eq!(report.scope, WorkspaceDirtyScope::GraphStructure);

    report.merge(WorkspaceDirtyScopeReport::new(
        WorkspaceDirtyScope::ConfigProject,
        vec![WorkspaceDirtyReason::ConfigChanged],
    ));
    assert_eq!(report.scope, WorkspaceDirtyScope::ConfigProject);

    report.merge(WorkspaceDirtyScopeReport::new(
        WorkspaceDirtyScope::Workspace,
        vec![WorkspaceDirtyReason::PackageGraphChanged],
    ));
    assert_eq!(report.scope, WorkspaceDirtyScope::Workspace);
    assert_eq!(
        report.reasons,
        vec![
            WorkspaceDirtyReason::ImportSignatureChanged,
            WorkspaceDirtyReason::ExportSignatureChanged,
            WorkspaceDirtyReason::FailedLookupChanged,
            WorkspaceDirtyReason::ConfigChanged,
            WorkspaceDirtyReason::PackageGraphChanged
        ]
    );
}

#[test]
fn single_file_session_freezes_overlay_state() {
    let path = SourcePath::new("scratch.sifr");
    let mut session = WorkspaceSession::single_file(path.clone(), FrontendMode::SingleFile);
    session.upsert_overlay(
        path,
        Some("file:///scratch.sifr".to_string()),
        DocumentVersion::new(1),
        SourceText::new("x = 1\n"),
        None,
    );
    session.reload().expect("single file overlay reloads");
    let first = session.snapshot();
    let second = session.snapshot();

    assert_eq!(first.id.as_u64(), 0);
    assert_eq!(second.id.as_u64(), 1);
    assert_eq!(first.overlays.len(), 1);
    assert!(Arc::ptr_eq(&first.overlays, &second.overlays));
    assert!(Arc::ptr_eq(
        &first.source_dependencies,
        &second.source_dependencies
    ));
    assert!(Arc::ptr_eq(
        first.source_map.as_ref().expect("first source map"),
        second.source_map.as_ref().expect("second source map")
    ));
    assert!(Arc::ptr_eq(
        first.module_graph.as_ref().expect("first graph"),
        second.module_graph.as_ref().expect("second graph")
    ));
    assert!(Arc::ptr_eq(
        &first.compiler_options,
        &second.compiler_options
    ));
    assert!(Arc::ptr_eq(
        &first.package_config_identity,
        &second.package_config_identity
    ));
    assert_eq!(first.compiler_options.mode, FrontendMode::SingleFile);
    assert!(first.source_dependencies.is_empty());
    assert!(first.module_graph.is_some());
}

struct TempProject {
    root: PathBuf,
}

impl TempProject {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sifr_frontend_{name}_{}_{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temp project");
        Self { root }
    }

    fn write(&self, relative: &str, source: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, source).expect("write source");
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(Path::new(&self.root));
    }
}
